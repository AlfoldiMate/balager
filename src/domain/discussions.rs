//! Discussions: threads, replies, votes, pins, polls, approver closing.

use super::{forbidden, invalid, not_found, require_approver, Actor, DomainResult};
use crate::backend::db::{db, now};
use crate::backend::notify::{self, Notice};
use crate::models::{MessageDto, PollDto, PollOptDto, ThreadDetailDto, ThreadDto};

#[derive(sqlx::FromRow)]
struct ThreadRow {
    id: i64,
    title: String,
    kind: String,
    link_id: Option<i64>,
    author_id: i64,
    closed: i64,
    created_at: i64,
}

async fn link_label(kind: &str, link_id: Option<i64>) -> String {
    let Some(link_id) = link_id else { return String::new() };
    let query = match kind {
        "reservation" => "SELECT title FROM reservations WHERE id = $1",
        "task" => "SELECT title FROM tasks WHERE id = $1",
        _ => return String::new(),
    };
    sqlx::query_scalar(query)
        .bind(link_id)
        .fetch_optional(db())
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
}

async fn thread_dto(row: ThreadRow) -> Result<ThreadDto, sqlx::Error> {
    let (replies, votes): (i64, i64) = sqlx::query_as(
        "SELECT
            (SELECT COUNT(*) FROM messages WHERE thread_id = $1 AND system = 0),
            (SELECT COALESCE(SUM(CASE WHEN v.value = 1 THEN 1 ELSE 0 END), 0)
             FROM message_votes v JOIN messages m ON m.id = v.message_id
             WHERE m.thread_id = $2)",
    )
    .bind(row.id)
    .bind(row.id)
    .fetch_one(db())
    .await?;
    let excerpt: Option<String> = sqlx::query_scalar(
        "SELECT body FROM messages
         WHERE thread_id = $1 AND system = 0 AND body != ''
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(row.id)
    .fetch_optional(db())
    .await?;
    let mut excerpt = excerpt.unwrap_or_default();
    if excerpt.chars().count() > 120 {
        excerpt = excerpt.chars().take(119).collect::<String>() + "…";
    }
    Ok(ThreadDto {
        link_label: link_label(&row.kind, row.link_id).await,
        id: row.id,
        title: row.title,
        kind: row.kind,
        link_id: row.link_id,
        author: row.author_id,
        created_at: row.created_at,
        closed: row.closed != 0,
        replies,
        votes,
        excerpt,
    })
}

#[derive(sqlx::FromRow)]
struct MsgRow {
    id: i64,
    parent_id: Option<i64>,
    author_id: Option<i64>,
    body: String,
    system: i64,
    pinned: i64,
    created_at: i64,
}

async fn messages_of(thread_id: i64, viewer: i64) -> Result<Vec<MessageDto>, sqlx::Error> {
    let rows: Vec<MsgRow> = sqlx::query_as(
        "SELECT id, parent_id, author_id, body, system, pinned, created_at
         FROM messages WHERE thread_id = $1 ORDER BY created_at, id",
    )
    .bind(thread_id)
    .fetch_all(db())
    .await?;
    let mut messages = Vec::new();
    for row in &rows {
        if row.parent_id.is_some() {
            continue;
        }
        let mut dto = msg_dto(row, viewer).await?;
        for child in rows.iter().filter(|r| r.parent_id == Some(row.id)) {
            dto.replies.push(msg_dto(child, viewer).await?);
        }
        messages.push(dto);
    }
    Ok(messages)
}

async fn msg_dto(row: &MsgRow, viewer: i64) -> Result<MessageDto, sqlx::Error> {
    let (up, down): (i64, i64) = sqlx::query_as(
        "SELECT COALESCE(SUM(CASE WHEN value = 1 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN value = -1 THEN 1 ELSE 0 END), 0)
         FROM message_votes WHERE message_id = $1",
    )
    .bind(row.id)
    .fetch_one(db())
    .await?;
    let my_vote: Option<i64> =
        sqlx::query_scalar("SELECT value FROM message_votes WHERE message_id = $1 AND user_id = $2")
            .bind(row.id)
            .bind(viewer)
            .fetch_optional(db())
            .await?;
    Ok(MessageDto {
        id: row.id,
        author: row.author_id,
        created_at: row.created_at,
        body: row.body.clone(),
        system: row.system != 0,
        pinned: row.pinned != 0,
        up,
        down,
        my_vote: my_vote.unwrap_or(0),
        poll: poll_of(row.id).await?,
        replies: Vec::new(),
    })
}

async fn poll_of(message_id: i64) -> Result<Option<PollDto>, sqlx::Error> {
    let poll: Option<(i64, String, String, String)> =
        sqlx::query_as("SELECT id, question, ptype, mode FROM polls WHERE message_id = $1")
            .bind(message_id)
            .fetch_optional(db())
            .await?;
    let Some((id, question, ptype, mode)) = poll else { return Ok(None) };
    let opts: Vec<(i64, String, String)> =
        sqlx::query_as("SELECT id, label, sub FROM poll_options WHERE poll_id = $1 ORDER BY id")
            .bind(id)
            .fetch_all(db())
            .await?;
    let mut options = Vec::new();
    for (oid, label, sub) in opts {
        let votes: Vec<(i64,)> = sqlx::query_as("SELECT user_id FROM poll_votes WHERE option_id = $1")
            .bind(oid)
            .fetch_all(db())
            .await?;
        options.push(PollOptDto {
            id: oid,
            label,
            sub,
            votes: votes.into_iter().map(|(u,)| u).collect(),
        });
    }
    Ok(Some(PollDto { id, question, ptype, mode, options }))
}

pub async fn list() -> DomainResult<Vec<ThreadDto>> {
    let rows: Vec<ThreadRow> = sqlx::query_as(
        "SELECT id, title, kind, link_id, author_id, closed, created_at
         FROM threads ORDER BY created_at DESC",
    )
    .fetch_all(db())
    .await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(thread_dto(row).await?);
    }
    Ok(out)
}

pub async fn get(viewer: &Actor, thread_id: i64) -> DomainResult<ThreadDetailDto> {
    let row: Option<ThreadRow> = sqlx::query_as(
        "SELECT id, title, kind, link_id, author_id, closed, created_at
         FROM threads WHERE id = $1",
    )
    .bind(thread_id)
    .fetch_optional(db())
    .await?;
    let Some(row) = row else {
        return Err(not_found("A beszélgetés nem található."));
    };
    Ok(ThreadDetailDto {
        thread: thread_dto(row).await?,
        messages: messages_of(thread_id, viewer.id).await?,
    })
}

/// New general discussion (linked threads are created from the reservation or
/// task side via [`open_or_create`]).
pub async fn create(actor: &Actor, title: String, body: String) -> DomainResult<i64> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(invalid("Adj címet a beszélgetésnek."));
    }
    let thread_id: i64 = sqlx::query_scalar(
        "INSERT INTO threads (title, kind, link_id, author_id, closed, created_at)
         VALUES ($1, 'general', NULL, $2, 0, $3) RETURNING id",
    )
    .bind(&title)
    .bind(actor.id)
    .bind(now())
    .fetch_one(db())
    .await?;
    let body = body.trim().to_string();
    if !body.is_empty() {
        sqlx::query(
            "INSERT INTO messages (thread_id, author_id, body, system, created_at)
             VALUES ($1, $2, $3, 0, $4)",
        )
        .bind(thread_id)
        .bind(actor.id)
        .bind(&body)
        .bind(now())
        .execute(db())
        .await?;
    }
    let others: Vec<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE active = 1 AND id != $1")
        .bind(actor.id)
        .fetch_all(db())
        .await?;
    notify::notify(
        &others.into_iter().map(|(id,)| id).collect::<Vec<_>>(),
        Notice {
            pref_key: "disc_new",
            icon: "chat",
            tone: "",
            text: format!("{} új beszélgetést indított: „{}”.", actor.name, title),
            link_kind: Some("thread"),
            link_id: Some(thread_id),
        },
    )
    .await;
    Ok(thread_id)
}

/// The discussion belonging to a reservation or task — created on first use.
pub async fn open_or_create(actor: &Actor, kind: String, link_id: i64) -> DomainResult<i64> {
    if !matches!(kind.as_str(), "reservation" | "task") {
        return Err(invalid("Érvénytelen hivatkozás."));
    }
    let existing: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM threads WHERE kind = $1 AND link_id = $2")
            .bind(&kind)
            .bind(link_id)
            .fetch_optional(db())
            .await?;
    if let Some((id,)) = existing {
        return Ok(id);
    }
    let title = link_label(&kind, Some(link_id)).await;
    if title.is_empty() {
        return Err(not_found("A hivatkozott elem nem található."));
    }
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO threads (title, kind, link_id, author_id, closed, created_at)
         VALUES ($1, $2, $3, $4, 0, $5) RETURNING id",
    )
    .bind(&title)
    .bind(&kind)
    .bind(link_id)
    .bind(actor.id)
    .bind(now())
    .fetch_one(db())
    .await?;
    Ok(id)
}

pub async fn post_message(
    actor: &Actor,
    thread_id: i64,
    parent_id: Option<i64>,
    body: String,
) -> DomainResult<i64> {
    let body = body.trim().to_string();
    if body.is_empty() {
        return Err(invalid("Üres üzenetet nem lehet küldeni."));
    }
    let thread: Option<(i64, String, i64)> =
        sqlx::query_as("SELECT author_id, title, closed FROM threads WHERE id = $1")
            .bind(thread_id)
            .fetch_optional(db())
            .await?;
    let Some((author_id, title, closed)) = thread else {
        return Err(not_found("A beszélgetés nem található."));
    };
    if closed != 0 {
        return Err(forbidden("Ezt a beszélgetést lezárták."));
    }
    // Replies only nest one level: replying to a reply attaches to its parent.
    let parent_id = match parent_id {
        Some(pid) => {
            let parent: Option<(Option<i64>,)> =
                sqlx::query_as("SELECT parent_id FROM messages WHERE id = $1 AND thread_id = $2")
                    .bind(pid)
                    .bind(thread_id)
                    .fetch_optional(db())
                    .await?;
            match parent {
                Some((Some(grandparent),)) => Some(grandparent),
                Some((None,)) => Some(pid),
                None => return Err(not_found("A válaszolt üzenet nem található.")),
            }
        }
        None => None,
    };
    let message_id: i64 = sqlx::query_scalar(
        "INSERT INTO messages (thread_id, parent_id, author_id, body, system, created_at)
         VALUES ($1, $2, $3, $4, 0, $5) RETURNING id",
    )
    .bind(thread_id)
    .bind(parent_id)
    .bind(actor.id)
    .bind(&body)
    .bind(now())
    .fetch_one(db())
    .await?;
    if author_id != actor.id {
        notify::notify(
            &[author_id],
            Notice {
                pref_key: "disc_reply",
                icon: "chat",
                tone: "",
                text: format!("{} új üzenetet írt: „{}”.", actor.name, title),
                link_kind: Some("thread"),
                link_id: Some(thread_id),
            },
        )
        .await;
    }
    Ok(message_id)
}

pub async fn set_pinned(message_id: i64, pinned: bool) -> DomainResult<()> {
    sqlx::query("UPDATE messages SET pinned = $1 WHERE id = $2 AND system = 0")
        .bind(pinned as i64)
        .bind(message_id)
        .execute(db())
        .await?;
    Ok(())
}

/// value: -1 (down), 0 (clear), 1 (up)
pub async fn vote(actor: &Actor, message_id: i64, value: i64) -> DomainResult<()> {
    if !matches!(value, -1 | 0 | 1) {
        return Err(invalid("Érvénytelen szavazat."));
    }
    if value == 0 {
        sqlx::query("DELETE FROM message_votes WHERE message_id = $1 AND user_id = $2")
            .bind(message_id)
            .bind(actor.id)
            .execute(db())
            .await?;
    } else {
        sqlx::query(
            "INSERT INTO message_votes (message_id, user_id, value) VALUES ($1, $2, $3)
             ON CONFLICT (message_id, user_id) DO UPDATE SET value = excluded.value",
        )
        .bind(message_id)
        .bind(actor.id)
        .bind(value)
        .execute(db())
        .await?;
    }
    Ok(())
}

/// Only approvers can close (or reopen) a discussion.
pub async fn set_closed(actor: &Actor, thread_id: i64, closed: bool) -> DomainResult<()> {
    require_approver(actor)?;
    sqlx::query("UPDATE threads SET closed = $1 WHERE id = $2")
        .bind(closed as i64)
        .bind(thread_id)
        .execute(db())
        .await?;
    let text = if closed {
        format!("A beszélgetést {} lezárta", actor.name)
    } else {
        format!("A beszélgetést {} újranyitotta", actor.name)
    };
    sqlx::query(
        "INSERT INTO messages (thread_id, author_id, body, system, created_at)
         VALUES ($1, NULL, $2, 1, $3)",
    )
    .bind(thread_id)
    .bind(&text)
    .bind(now())
    .execute(db())
    .await?;
    Ok(())
}

pub async fn delete(actor: &Actor, thread_id: i64) -> DomainResult<()> {
    let author: Option<(i64,)> = sqlx::query_as("SELECT author_id FROM threads WHERE id = $1")
        .bind(thread_id)
        .fetch_optional(db())
        .await?;
    let Some((author_id,)) = author else {
        return Err(not_found("A beszélgetés nem található."));
    };
    if author_id != actor.id && actor.role != "approver" {
        return Err(forbidden("Csak a téma indítója vagy egy engedélyező törölheti."));
    }
    sqlx::query("DELETE FROM threads WHERE id = $1")
        .bind(thread_id)
        .execute(db())
        .await?;
    Ok(())
}

pub async fn create_poll(
    actor: &Actor,
    thread_id: i64,
    question: String,
    ptype: String,
    mode: String,
    options: Vec<String>,
) -> DomainResult<i64> {
    let question = question.trim().to_string();
    let options: Vec<String> = options
        .into_iter()
        .map(|o| o.trim().to_string())
        .filter(|o| !o.is_empty())
        .collect();
    if question.is_empty() || options.len() < 2 {
        return Err(invalid("Adj meg kérdést és legalább két opciót."));
    }
    if !matches!(ptype.as_str(), "date" | "list") || !matches!(mode.as_str(), "single" | "multi") {
        return Err(invalid("Érvénytelen szavazás-típus."));
    }
    let closed: i64 = sqlx::query_scalar("SELECT closed FROM threads WHERE id = $1")
        .bind(thread_id)
        .fetch_one(db())
        .await?;
    if closed != 0 {
        return Err(forbidden("Ezt a beszélgetést lezárták."));
    }
    let message_id: i64 = sqlx::query_scalar(
        "INSERT INTO messages (thread_id, author_id, body, system, created_at)
         VALUES ($1, $2, '', 0, $3) RETURNING id",
    )
    .bind(thread_id)
    .bind(actor.id)
    .bind(now())
    .fetch_one(db())
    .await?;
    let poll_id: i64 = sqlx::query_scalar(
        "INSERT INTO polls (message_id, question, ptype, mode) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(message_id)
    .bind(&question)
    .bind(&ptype)
    .bind(&mode)
    .fetch_one(db())
    .await?;
    for option in &options {
        sqlx::query("INSERT INTO poll_options (poll_id, label) VALUES ($1, $2)")
            .bind(poll_id)
            .bind(option)
            .execute(db())
            .await?;
    }
    Ok(message_id)
}

pub async fn poll_vote(actor: &Actor, option_id: i64, on: bool) -> DomainResult<()> {
    let poll: Option<(i64, String)> = sqlx::query_as(
        "SELECT p.id, p.mode FROM polls p JOIN poll_options o ON o.poll_id = p.id WHERE o.id = $1",
    )
    .bind(option_id)
    .fetch_optional(db())
    .await?;
    let Some((poll_id, mode)) = poll else {
        return Err(not_found("A szavazás nem található."));
    };
    if on {
        if mode == "single" {
            sqlx::query(
                "DELETE FROM poll_votes WHERE user_id = $1
                 AND option_id IN (SELECT id FROM poll_options WHERE poll_id = $2)",
            )
            .bind(actor.id)
            .bind(poll_id)
            .execute(db())
            .await?;
        }
        sqlx::query(
            "INSERT INTO poll_votes (option_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(option_id)
        .bind(actor.id)
        .execute(db())
        .await?;
    } else {
        sqlx::query("DELETE FROM poll_votes WHERE option_id = $1 AND user_id = $2")
            .bind(option_id)
            .bind(actor.id)
            .execute(db())
            .await?;
    }
    Ok(())
}

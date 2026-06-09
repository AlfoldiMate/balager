//! Discussions: threads, replies, votes, pins, polls, approver closing.

use dioxus::prelude::*;

use crate::models::{ThreadDetailDto, ThreadDto};

#[cfg(feature = "server")]
fn err(e: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::new(e.to_string())
}

#[cfg(feature = "server")]
mod server {
    use crate::backend::db::db;
    use crate::models::{MessageDto, PollDto, PollOptDto, ThreadDto};

    #[derive(sqlx::FromRow)]
    pub struct ThreadRow {
        pub id: i64,
        pub title: String,
        pub kind: String,
        pub link_id: Option<i64>,
        pub author_id: i64,
        pub closed: i64,
        pub created_at: i64,
    }

    pub async fn link_label(kind: &str, link_id: Option<i64>) -> String {
        let Some(link_id) = link_id else { return String::new() };
        let query = match kind {
            "reservation" => "SELECT title FROM reservations WHERE id = ?",
            "task" => "SELECT title FROM tasks WHERE id = ?",
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

    pub async fn thread_dto(row: ThreadRow) -> Result<ThreadDto, sqlx::Error> {
        let (replies, votes): (i64, i64) = sqlx::query_as(
            "SELECT
                (SELECT COUNT(*) FROM messages WHERE thread_id = ? AND system = 0),
                (SELECT COALESCE(SUM(CASE WHEN v.value = 1 THEN 1 ELSE 0 END), 0)
                 FROM message_votes v JOIN messages m ON m.id = v.message_id
                 WHERE m.thread_id = ?)",
        )
        .bind(row.id)
        .bind(row.id)
        .fetch_one(db())
        .await?;
        let excerpt: Option<String> = sqlx::query_scalar(
            "SELECT body FROM messages
             WHERE thread_id = ? AND system = 0 AND body != ''
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

    pub async fn messages_of(thread_id: i64, viewer: i64) -> Result<Vec<MessageDto>, sqlx::Error> {
        let rows: Vec<MsgRow> = sqlx::query_as(
            "SELECT id, parent_id, author_id, body, system, pinned, created_at
             FROM messages WHERE thread_id = ? ORDER BY created_at, id",
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
             FROM message_votes WHERE message_id = ?",
        )
        .bind(row.id)
        .fetch_one(db())
        .await?;
        let my_vote: Option<i64> =
            sqlx::query_scalar("SELECT value FROM message_votes WHERE message_id = ? AND user_id = ?")
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
        let poll: Option<(i64, String, String, String)> = sqlx::query_as(
            "SELECT id, question, ptype, mode FROM polls WHERE message_id = ?",
        )
        .bind(message_id)
        .fetch_optional(db())
        .await?;
        let Some((id, question, ptype, mode)) = poll else { return Ok(None) };
        let opts: Vec<(i64, String, String)> =
            sqlx::query_as("SELECT id, label, sub FROM poll_options WHERE poll_id = ? ORDER BY id")
                .bind(id)
                .fetch_all(db())
                .await?;
        let mut options = Vec::new();
        for (oid, label, sub) in opts {
            let votes: Vec<(i64,)> =
                sqlx::query_as("SELECT user_id FROM poll_votes WHERE option_id = ?")
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
}

#[server]
pub async fn list_threads() -> Result<Vec<ThreadDto>, ServerFnError> {
    use crate::backend::{auth, db::db};

    auth::require_user().await?;
    let rows: Vec<server::ThreadRow> = sqlx::query_as(
        "SELECT id, title, kind, link_id, author_id, closed, created_at
         FROM threads ORDER BY created_at DESC",
    )
    .fetch_all(db())
    .await
    .map_err(err)?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(server::thread_dto(row).await.map_err(err)?);
    }
    Ok(out)
}

#[server]
pub async fn get_thread(thread_id: i64) -> Result<ThreadDetailDto, ServerFnError> {
    use crate::backend::{auth, db::db};

    let user = auth::require_user().await?;
    let row: Option<server::ThreadRow> = sqlx::query_as(
        "SELECT id, title, kind, link_id, author_id, closed, created_at
         FROM threads WHERE id = ?",
    )
    .bind(thread_id)
    .fetch_optional(db())
    .await
    .map_err(err)?;
    let Some(row) = row else {
        return Err(ServerFnError::new("A beszélgetés nem található."));
    };
    Ok(ThreadDetailDto {
        thread: server::thread_dto(row).await.map_err(err)?,
        messages: server::messages_of(thread_id, user.id).await.map_err(err)?,
    })
}

/// New general discussion (linked threads are created from the reservation or
/// task side via [`open_or_create_thread`]).
#[server]
pub async fn create_thread(title: String, body: String) -> Result<i64, ServerFnError> {
    use crate::backend::{auth, db::{db, now}, notify};

    let user = auth::require_user().await?;
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(ServerFnError::new("Adj címet a beszélgetésnek."));
    }
    let result = sqlx::query(
        "INSERT INTO threads (title, kind, link_id, author_id, closed, created_at)
         VALUES (?, 'general', NULL, ?, 0, ?)",
    )
    .bind(&title)
    .bind(user.id)
    .bind(now())
    .execute(db())
    .await
    .map_err(err)?;
    let thread_id = result.last_insert_rowid();
    let body = body.trim().to_string();
    if !body.is_empty() {
        sqlx::query(
            "INSERT INTO messages (thread_id, author_id, body, system, created_at)
             VALUES (?, ?, ?, 0, ?)",
        )
        .bind(thread_id)
        .bind(user.id)
        .bind(&body)
        .bind(now())
        .execute(db())
        .await
        .map_err(err)?;
    }
    let others: Vec<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE active = 1 AND id != ?")
        .bind(user.id)
        .fetch_all(db())
        .await
        .map_err(err)?;
    notify::notify(
        &others.into_iter().map(|(id,)| id).collect::<Vec<_>>(),
        notify::Notice {
            pref_key: "disc_new",
            icon: "chat",
            tone: "",
            text: format!("{} új beszélgetést indított: „{}”.", user.name, title),
            link_kind: Some("thread"),
            link_id: Some(thread_id),
        },
    )
    .await;
    Ok(thread_id)
}

/// The discussion belonging to a reservation or task — created on first use.
#[server]
pub async fn open_or_create_thread(kind: String, link_id: i64) -> Result<i64, ServerFnError> {
    use crate::backend::{auth, db::{db, now}};

    let user = auth::require_user().await?;
    if !matches!(kind.as_str(), "reservation" | "task") {
        return Err(ServerFnError::new("Érvénytelen hivatkozás."));
    }
    let existing: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM threads WHERE kind = ? AND link_id = ?")
            .bind(&kind)
            .bind(link_id)
            .fetch_optional(db())
            .await
            .map_err(err)?;
    if let Some((id,)) = existing {
        return Ok(id);
    }
    let title = server::link_label(&kind, Some(link_id)).await;
    if title.is_empty() {
        return Err(ServerFnError::new("A hivatkozott elem nem található."));
    }
    let result = sqlx::query(
        "INSERT INTO threads (title, kind, link_id, author_id, closed, created_at)
         VALUES (?, ?, ?, ?, 0, ?)",
    )
    .bind(&title)
    .bind(&kind)
    .bind(link_id)
    .bind(user.id)
    .bind(now())
    .execute(db())
    .await
    .map_err(err)?;
    Ok(result.last_insert_rowid())
}

#[server]
pub async fn post_message(
    thread_id: i64,
    parent_id: Option<i64>,
    body: String,
) -> Result<i64, ServerFnError> {
    use crate::backend::{auth, db::{db, now}, notify};

    let user = auth::require_user().await?;
    let body = body.trim().to_string();
    if body.is_empty() {
        return Err(ServerFnError::new("Üres üzenetet nem lehet küldeni."));
    }
    let thread: Option<(i64, String, i64)> =
        sqlx::query_as("SELECT author_id, title, closed FROM threads WHERE id = ?")
            .bind(thread_id)
            .fetch_optional(db())
            .await
            .map_err(err)?;
    let Some((author_id, title, closed)) = thread else {
        return Err(ServerFnError::new("A beszélgetés nem található."));
    };
    if closed != 0 {
        return Err(ServerFnError::new("Ezt a beszélgetést lezárták."));
    }
    // Replies only nest one level: replying to a reply attaches to its parent.
    let parent_id = match parent_id {
        Some(pid) => {
            let parent: Option<(Option<i64>,)> =
                sqlx::query_as("SELECT parent_id FROM messages WHERE id = ? AND thread_id = ?")
                    .bind(pid)
                    .bind(thread_id)
                    .fetch_optional(db())
                    .await
                    .map_err(err)?;
            match parent {
                Some((Some(grandparent),)) => Some(grandparent),
                Some((None,)) => Some(pid),
                None => return Err(ServerFnError::new("A válaszolt üzenet nem található.")),
            }
        }
        None => None,
    };
    let result = sqlx::query(
        "INSERT INTO messages (thread_id, parent_id, author_id, body, system, created_at)
         VALUES (?, ?, ?, ?, 0, ?)",
    )
    .bind(thread_id)
    .bind(parent_id)
    .bind(user.id)
    .bind(&body)
    .bind(now())
    .execute(db())
    .await
    .map_err(err)?;
    if author_id != user.id {
        notify::notify(
            &[author_id],
            notify::Notice {
                pref_key: "disc_reply",
                icon: "chat",
                tone: "",
                text: format!("{} új üzenetet írt: „{}”.", user.name, title),
                link_kind: Some("thread"),
                link_id: Some(thread_id),
            },
        )
        .await;
    }
    Ok(result.last_insert_rowid())
}

#[server]
pub async fn set_pinned(message_id: i64, pinned: bool) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};

    auth::require_user().await?;
    sqlx::query("UPDATE messages SET pinned = ? WHERE id = ? AND system = 0")
        .bind(pinned as i64)
        .bind(message_id)
        .execute(db())
        .await
        .map_err(err)?;
    Ok(())
}

/// value: -1 (down), 0 (clear), 1 (up)
#[server]
pub async fn vote_message(message_id: i64, value: i64) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};

    let user = auth::require_user().await?;
    if !matches!(value, -1 | 0 | 1) {
        return Err(ServerFnError::new("Érvénytelen szavazat."));
    }
    if value == 0 {
        sqlx::query("DELETE FROM message_votes WHERE message_id = ? AND user_id = ?")
            .bind(message_id)
            .bind(user.id)
            .execute(db())
            .await
            .map_err(err)?;
    } else {
        sqlx::query(
            "INSERT INTO message_votes (message_id, user_id, value) VALUES (?, ?, ?)
             ON CONFLICT (message_id, user_id) DO UPDATE SET value = excluded.value",
        )
        .bind(message_id)
        .bind(user.id)
        .bind(value)
        .execute(db())
        .await
        .map_err(err)?;
    }
    Ok(())
}

/// Only approvers can close (or reopen) a discussion.
#[server]
pub async fn set_thread_closed(thread_id: i64, closed: bool) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::{db, now}};

    let approver = auth::require_approver().await?;
    sqlx::query("UPDATE threads SET closed = ? WHERE id = ?")
        .bind(closed as i64)
        .bind(thread_id)
        .execute(db())
        .await
        .map_err(err)?;
    let text = if closed {
        format!("A beszélgetést {} lezárta", approver.name)
    } else {
        format!("A beszélgetést {} újranyitotta", approver.name)
    };
    sqlx::query(
        "INSERT INTO messages (thread_id, author_id, body, system, created_at)
         VALUES (?, NULL, ?, 1, ?)",
    )
    .bind(thread_id)
    .bind(&text)
    .bind(now())
    .execute(db())
    .await
    .map_err(err)?;
    Ok(())
}

#[server]
pub async fn delete_thread(thread_id: i64) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};

    let user = auth::require_user().await?;
    let author: Option<(i64,)> = sqlx::query_as("SELECT author_id FROM threads WHERE id = ?")
        .bind(thread_id)
        .fetch_optional(db())
        .await
        .map_err(err)?;
    let Some((author_id,)) = author else {
        return Err(ServerFnError::new("A beszélgetés nem található."));
    };
    if author_id != user.id && user.role != "approver" {
        return Err(ServerFnError::new("Csak a téma indítója vagy egy engedélyező törölheti."));
    }
    sqlx::query("DELETE FROM threads WHERE id = ?")
        .bind(thread_id)
        .execute(db())
        .await
        .map_err(err)?;
    Ok(())
}

#[server]
pub async fn create_poll(
    thread_id: i64,
    question: String,
    ptype: String,
    mode: String,
    options: Vec<String>,
) -> Result<i64, ServerFnError> {
    use crate::backend::{auth, db::{db, now}};

    let user = auth::require_user().await?;
    let question = question.trim().to_string();
    let options: Vec<String> = options
        .into_iter()
        .map(|o| o.trim().to_string())
        .filter(|o| !o.is_empty())
        .collect();
    if question.is_empty() || options.len() < 2 {
        return Err(ServerFnError::new("Adj meg kérdést és legalább két opciót."));
    }
    if !matches!(ptype.as_str(), "date" | "list") || !matches!(mode.as_str(), "single" | "multi") {
        return Err(ServerFnError::new("Érvénytelen szavazás-típus."));
    }
    let closed: i64 = sqlx::query_scalar("SELECT closed FROM threads WHERE id = ?")
        .bind(thread_id)
        .fetch_one(db())
        .await
        .map_err(err)?;
    if closed != 0 {
        return Err(ServerFnError::new("Ezt a beszélgetést lezárták."));
    }
    let result = sqlx::query(
        "INSERT INTO messages (thread_id, author_id, body, system, created_at)
         VALUES (?, ?, '', 0, ?)",
    )
    .bind(thread_id)
    .bind(user.id)
    .bind(now())
    .execute(db())
    .await
    .map_err(err)?;
    let message_id = result.last_insert_rowid();
    let result = sqlx::query("INSERT INTO polls (message_id, question, ptype, mode) VALUES (?, ?, ?, ?)")
        .bind(message_id)
        .bind(&question)
        .bind(&ptype)
        .bind(&mode)
        .execute(db())
        .await
        .map_err(err)?;
    let poll_id = result.last_insert_rowid();
    for option in &options {
        sqlx::query("INSERT INTO poll_options (poll_id, label) VALUES (?, ?)")
            .bind(poll_id)
            .bind(option)
            .execute(db())
            .await
            .map_err(err)?;
    }
    Ok(message_id)
}

#[server]
pub async fn poll_vote(option_id: i64, on: bool) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};

    let user = auth::require_user().await?;
    let poll: Option<(i64, String)> = sqlx::query_as(
        "SELECT p.id, p.mode FROM polls p JOIN poll_options o ON o.poll_id = p.id WHERE o.id = ?",
    )
    .bind(option_id)
    .fetch_optional(db())
    .await
    .map_err(err)?;
    let Some((poll_id, mode)) = poll else {
        return Err(ServerFnError::new("A szavazás nem található."));
    };
    if on {
        if mode == "single" {
            sqlx::query(
                "DELETE FROM poll_votes WHERE user_id = ?
                 AND option_id IN (SELECT id FROM poll_options WHERE poll_id = ?)",
            )
            .bind(user.id)
            .bind(poll_id)
            .execute(db())
            .await
            .map_err(err)?;
        }
        sqlx::query("INSERT OR IGNORE INTO poll_votes (option_id, user_id) VALUES (?, ?)")
            .bind(option_id)
            .bind(user.id)
            .execute(db())
            .await
            .map_err(err)?;
    } else {
        sqlx::query("DELETE FROM poll_votes WHERE option_id = ? AND user_id = ?")
            .bind(option_id)
            .bind(user.id)
            .execute(db())
            .await
            .map_err(err)?;
    }
    Ok(())
}

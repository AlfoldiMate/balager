//! Reservations: request → approvals → approved/rejected; open/closed access;
//! attendees; propagation of changes into the linked discussion thread.

use super::{forbidden, invalid, not_found, require_approver, Actor, DomainResult};
use crate::backend::db::{db, now};
use crate::backend::notify::{self, Notice};
use crate::models::{hu_range, parse_iso, ApprovalDto, NewReservation, ReservationDto};

#[derive(sqlx::FromRow)]
struct ResRow {
    id: i64,
    title: String,
    day_from: String,
    day_to: String,
    owner_id: i64,
    access: String,
    note: String,
}

/// Derive the UI status from the approvals: any rejection wins, full approval
/// surfaces the access mode, anything else is pending.
fn derive_status(access: &str, approvals: &[ApprovalDto]) -> String {
    if approvals.iter().any(|a| a.status == "rejected") {
        return "reject".into();
    }
    if !approvals.is_empty() && approvals.iter().all(|a| a.status == "approved") {
        return access.to_string();
    }
    "pending".into()
}

async fn assemble(row: ResRow) -> Result<ReservationDto, sqlx::Error> {
    let approvals: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT approver_id, status, comment FROM reservation_approvals
         WHERE reservation_id = $1 ORDER BY approver_id",
    )
    .bind(row.id)
    .fetch_all(db())
    .await?;
    let approvals: Vec<ApprovalDto> = approvals
        .into_iter()
        .map(|(user_id, status, comment)| ApprovalDto { user_id, status, comment })
        .collect();
    let attendees: Vec<(i64,)> = sqlx::query_as(
        "SELECT user_id FROM reservation_attendees WHERE reservation_id = $1 ORDER BY user_id",
    )
    .bind(row.id)
    .fetch_all(db())
    .await?;
    let thread_id: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM threads WHERE kind = 'reservation' AND link_id = $1")
            .bind(row.id)
            .fetch_optional(db())
            .await?;
    let rejecting = approvals.iter().find(|a| a.status == "rejected");
    Ok(ReservationDto {
        id: row.id,
        title: row.title,
        from: row.day_from,
        to: row.day_to,
        status: derive_status(&row.access, &approvals),
        access: row.access,
        owner: row.owner_id,
        attendees: attendees.into_iter().map(|(id,)| id).collect(),
        reject_reason: rejecting.map(|a| a.comment.clone()).unwrap_or_default(),
        rejected_by: rejecting.map(|a| a.user_id),
        approvals,
        note: row.note,
        thread_id: thread_id.map(|(id,)| id),
    })
}

async fn fetch(res_id: i64) -> DomainResult<ReservationDto> {
    let row: Option<ResRow> = sqlx::query_as(
        "SELECT id, title, day_from, day_to, owner_id, access, note
         FROM reservations WHERE id = $1",
    )
    .bind(res_id)
    .fetch_optional(db())
    .await?;
    match row {
        Some(row) => Ok(assemble(row).await?),
        None => Err(not_found("A foglalás nem található.")),
    }
}

/// Insert a system message into the thread linked to a reservation, if any.
async fn system_message(res_id: i64, text: &str) {
    let thread: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM threads WHERE kind = 'reservation' AND link_id = $1")
            .bind(res_id)
            .fetch_optional(db())
            .await
            .ok()
            .flatten();
    if let Some((thread_id,)) = thread {
        let _ = sqlx::query(
            "INSERT INTO messages (thread_id, author_id, body, system, created_at)
             VALUES ($1, NULL, $2, 1, $3)",
        )
        .bind(thread_id)
        .bind(text)
        .bind(now())
        .execute(db())
        .await;
    }
}

pub async fn list() -> DomainResult<Vec<ReservationDto>> {
    let rows: Vec<ResRow> = sqlx::query_as(
        "SELECT id, title, day_from, day_to, owner_id, access, note
         FROM reservations ORDER BY day_from",
    )
    .fetch_all(db())
    .await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(assemble(row).await?);
    }
    Ok(out)
}

pub async fn create(actor: &Actor, input: NewReservation) -> DomainResult<i64> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(invalid("Adj nevet a foglalásnak."));
    }
    let (Some(from), Some(to)) = (parse_iso(&input.from), parse_iso(&input.to)) else {
        return Err(invalid("Érvénytelen dátum."));
    };
    if to < from {
        return Err(invalid("A záró nap nem lehet a kezdőnap előtt."));
    }
    if !matches!(input.access.as_str(), "closed" | "open") {
        return Err(invalid("Érvénytelen hozzáférés."));
    }

    // Only unreserved days can be requested: overlap with any non-rejected
    // reservation is refused.
    let overlapping: Vec<ResRow> = sqlx::query_as(
        "SELECT id, title, day_from, day_to, owner_id, access, note FROM reservations
         WHERE day_from <= $1 AND day_to >= $2",
    )
    .bind(&input.to)
    .bind(&input.from)
    .fetch_all(db())
    .await?;
    for row in overlapping {
        let dto = assemble(row).await?;
        if dto.status != "reject" {
            return Err(invalid(format!(
                "Ütközés: „{}” ({}).",
                dto.title,
                hu_range(&dto.from, &dto.to)
            )));
        }
    }

    let res_id: i64 = sqlx::query_scalar(
        "INSERT INTO reservations (title, day_from, day_to, owner_id, access, note, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
    )
    .bind(&title)
    .bind(&input.from)
    .bind(&input.to)
    .bind(actor.id)
    .bind(&input.access)
    .bind(input.note.trim())
    .bind(now())
    .fetch_one(db())
    .await?;

    sqlx::query("INSERT INTO reservation_attendees (reservation_id, user_id) VALUES ($1, $2)")
        .bind(res_id)
        .bind(actor.id)
        .execute(db())
        .await?;

    // One approval row per active approver; requesting your own reservation
    // counts as your approval if you are an approver yourself.
    let approvers: Vec<(i64,)> =
        sqlx::query_as("SELECT id FROM users WHERE role = 'approver' AND active = 1")
            .fetch_all(db())
            .await?;
    for (approver_id,) in &approvers {
        let own = *approver_id == actor.id;
        sqlx::query(
            "INSERT INTO reservation_approvals (reservation_id, approver_id, status, decided_at)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(res_id)
        .bind(approver_id)
        .bind(if own { "approved" } else { "pending" })
        .bind(if own { Some(now()) } else { None })
        .execute(db())
        .await?;
    }

    let recipients: Vec<i64> = approvers
        .iter()
        .map(|(id,)| *id)
        .filter(|id| *id != actor.id)
        .collect();
    notify::notify(
        &recipients,
        Notice {
            pref_key: "res_request",
            icon: "clock",
            tone: "pending",
            text: format!(
                "{} foglalást kért: „{}” ({}).",
                actor.name,
                title,
                hu_range(&input.from, &input.to)
            ),
            link_kind: Some("reservation"),
            link_id: Some(res_id),
        },
    )
    .await;

    Ok(res_id)
}

/// Approve or reject a reservation. Rejection requires a comment.
pub async fn decide(actor: &Actor, res_id: i64, approve: bool, comment: String) -> DomainResult<()> {
    require_approver(actor)?;
    let comment = comment.trim().to_string();
    if !approve && comment.is_empty() {
        return Err(invalid("Elutasításhoz indoklás szükséges."));
    }
    let dto = fetch(res_id).await?;

    sqlx::query(
        "INSERT INTO reservation_approvals (reservation_id, approver_id, status, comment, decided_at)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (reservation_id, approver_id)
         DO UPDATE SET status = excluded.status, comment = excluded.comment, decided_at = excluded.decided_at",
    )
    .bind(res_id)
    .bind(actor.id)
    .bind(if approve { "approved" } else { "rejected" })
    .bind(&comment)
    .bind(now())
    .execute(db())
    .await?;

    system_message(
        res_id,
        &if approve {
            format!("{} jóváhagyta a foglalást", actor.name)
        } else {
            format!("{} elutasította a foglalást: „{}”", actor.name, comment)
        },
    )
    .await;
    if dto.owner != actor.id {
        let text = if approve {
            format!("{} jóváhagyta a(z) „{}” foglalást.", actor.name, dto.title)
        } else {
            format!(
                "{} elutasította a(z) „{}” foglalást: „{}”",
                actor.name, dto.title, comment
            )
        };
        notify::notify(
            &[dto.owner],
            Notice {
                pref_key: "res_decision",
                icon: if approve { "check" } else { "x" },
                tone: if approve { "open" } else { "reject" },
                text,
                link_kind: Some("reservation"),
                link_id: Some(res_id),
            },
        )
        .await;
    }
    Ok(())
}

pub async fn set_access(actor: &Actor, res_id: i64, access: String) -> DomainResult<()> {
    if !matches!(access.as_str(), "closed" | "open") {
        return Err(invalid("Érvénytelen hozzáférés."));
    }
    let dto = fetch(res_id).await?;
    if dto.owner != actor.id {
        return Err(forbidden("A hozzáférést csak a foglaló módosíthatja."));
    }
    sqlx::query("UPDATE reservations SET access = $1 WHERE id = $2")
        .bind(&access)
        .bind(res_id)
        .execute(db())
        .await?;
    system_message(
        res_id,
        &if access == "open" {
            format!("{} nyitottá tette a foglalást — bárki csatlakozhat", actor.name)
        } else {
            format!("{} zárttá tette a foglalást", actor.name)
        },
    )
    .await;
    Ok(())
}

/// Join an open reservation (or leave it again).
pub async fn set_attendance(actor: &Actor, res_id: i64, attend: bool) -> DomainResult<()> {
    let dto = fetch(res_id).await?;
    if dto.owner == actor.id {
        return Err(invalid("A foglaló mindig résztvevő."));
    }
    if dto.access != "open" {
        return Err(forbidden("Zárt foglaláshoz csak a foglaló adhat hozzá résztvevőt."));
    }
    if dto.status == "reject" {
        return Err(invalid("Elutasított foglaláshoz nem lehet csatlakozni."));
    }
    if attend {
        sqlx::query(
            "INSERT INTO reservation_attendees (reservation_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(res_id)
        .bind(actor.id)
        .execute(db())
        .await?;
        system_message(res_id, &format!("{} csatlakozott a foglaláshoz", actor.name)).await;
        notify::notify(
            &[dto.owner],
            Notice {
                pref_key: "res_join",
                icon: "users",
                tone: "closed",
                text: format!(
                    "{} csatlakozott a(z) „{}” nyitott foglaláshoz.",
                    actor.name, dto.title
                ),
                link_kind: Some("reservation"),
                link_id: Some(res_id),
            },
        )
        .await;
    } else {
        sqlx::query("DELETE FROM reservation_attendees WHERE reservation_id = $1 AND user_id = $2")
            .bind(res_id)
            .bind(actor.id)
            .execute(db())
            .await?;
        system_message(res_id, &format!("{} lemondta a részvételt", actor.name)).await;
    }
    Ok(())
}

/// Owner manages the attendee list (closed reservations, or corrections).
pub async fn set_attendee(actor: &Actor, res_id: i64, user_id: i64, attend: bool) -> DomainResult<()> {
    let dto = fetch(res_id).await?;
    if dto.owner != actor.id {
        return Err(forbidden("A résztvevőket csak a foglaló kezelheti."));
    }
    if user_id == dto.owner && !attend {
        return Err(invalid("A foglaló mindig résztvevő."));
    }
    let target: Option<(String,)> =
        sqlx::query_as("SELECT name FROM users WHERE id = $1 AND active = 1")
            .bind(user_id)
            .fetch_optional(db())
            .await?;
    let Some((target_name,)) = target else {
        return Err(not_found("A felhasználó nem található."));
    };
    if attend {
        sqlx::query(
            "INSERT INTO reservation_attendees (reservation_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(res_id)
        .bind(user_id)
        .execute(db())
        .await?;
        system_message(res_id, &format!("{} hozzáadta {} résztvevőt", actor.name, target_name))
            .await;
    } else {
        sqlx::query("DELETE FROM reservation_attendees WHERE reservation_id = $1 AND user_id = $2")
            .bind(res_id)
            .bind(user_id)
            .execute(db())
            .await?;
        system_message(
            res_id,
            &format!("{} eltávolította {} résztvevőt", actor.name, target_name),
        )
        .await;
    }
    Ok(())
}

/// Cancel a reservation (owner, or an approver).
pub async fn delete(actor: &Actor, res_id: i64) -> DomainResult<()> {
    let dto = fetch(res_id).await?;
    if dto.owner != actor.id && actor.role != "approver" {
        return Err(forbidden("Csak a foglaló vagy egy engedélyező törölheti."));
    }
    system_message(res_id, &format!("{} törölte a(z) „{}” foglalást", actor.name, dto.title))
        .await;
    sqlx::query("DELETE FROM reservations WHERE id = $1")
        .bind(res_id)
        .execute(db())
        .await?;
    let recipients: Vec<i64> = dto
        .attendees
        .iter()
        .copied()
        .filter(|id| *id != actor.id)
        .collect();
    notify::notify(
        &recipients,
        Notice {
            pref_key: "res_decision",
            icon: "x",
            tone: "reject",
            text: format!(
                "{} törölte a(z) „{}” foglalást ({}).",
                actor.name,
                dto.title,
                hu_range(&dto.from, &dto.to)
            ),
            link_kind: None,
            link_id: None,
        },
    )
    .await;
    Ok(())
}

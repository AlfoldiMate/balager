//! Reservations: request → approvals → approved/rejected; open/closed access;
//! attendees; propagation of changes into the linked discussion thread.

use dioxus::prelude::*;

use crate::models::{NewReservation, ReservationDto};

#[cfg(feature = "server")]
fn err(e: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::new(e.to_string())
}

#[cfg(feature = "server")]
mod server {
    use crate::backend::db::{db, now};
    use crate::models::{hu_range, ApprovalDto, ReservationDto};

    #[derive(sqlx::FromRow)]
    pub struct ResRow {
        pub id: i64,
        pub title: String,
        pub day_from: String,
        pub day_to: String,
        pub owner_id: i64,
        pub access: String,
        pub note: String,
    }

    /// Derive the UI status from the approvals.
    pub fn derive_status(access: &str, approvals: &[ApprovalDto]) -> String {
        if approvals.iter().any(|a| a.status == "rejected") {
            return "reject".into();
        }
        if !approvals.is_empty() && approvals.iter().all(|a| a.status == "approved") {
            return access.to_string();
        }
        "pending".into()
    }

    pub async fn assemble(row: ResRow) -> Result<ReservationDto, sqlx::Error> {
        let approvals: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT approver_id, status, comment FROM reservation_approvals
             WHERE reservation_id = ? ORDER BY approver_id",
        )
        .bind(row.id)
        .fetch_all(db())
        .await?;
        let approvals: Vec<ApprovalDto> = approvals
            .into_iter()
            .map(|(user_id, status, comment)| ApprovalDto { user_id, status, comment })
            .collect();
        let attendees: Vec<(i64,)> = sqlx::query_as(
            "SELECT user_id FROM reservation_attendees WHERE reservation_id = ? ORDER BY user_id",
        )
        .bind(row.id)
        .fetch_all(db())
        .await?;
        let thread_id: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM threads WHERE kind = 'reservation' AND link_id = ?",
        )
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

    pub async fn fetch(res_id: i64) -> Result<Option<ReservationDto>, sqlx::Error> {
        let row: Option<ResRow> = sqlx::query_as(
            "SELECT id, title, day_from, day_to, owner_id, access, note
             FROM reservations WHERE id = ?",
        )
        .bind(res_id)
        .fetch_optional(db())
        .await?;
        match row {
            Some(row) => Ok(Some(assemble(row).await?)),
            None => Ok(None),
        }
    }

    /// Insert a system message into the thread linked to a reservation, if any.
    pub async fn system_message(res_id: i64, text: &str) {
        let thread: Option<(i64,)> =
            sqlx::query_as("SELECT id FROM threads WHERE kind = 'reservation' AND link_id = ?")
                .bind(res_id)
                .fetch_optional(db())
                .await
                .ok()
                .flatten();
        if let Some((thread_id,)) = thread {
            let _ = sqlx::query(
                "INSERT INTO messages (thread_id, author_id, body, system, created_at)
                 VALUES (?, NULL, ?, 1, ?)",
            )
            .bind(thread_id)
            .bind(text)
            .bind(now())
            .execute(db())
            .await;
        }
    }

    pub fn range_label(dto: &ReservationDto) -> String {
        hu_range(&dto.from, &dto.to)
    }
}

#[server]
pub async fn list_reservations() -> Result<Vec<ReservationDto>, ServerFnError> {
    use crate::backend::{auth, db::db};

    auth::require_user().await?;
    let rows: Vec<server::ResRow> = sqlx::query_as(
        "SELECT id, title, day_from, day_to, owner_id, access, note
         FROM reservations ORDER BY day_from",
    )
    .fetch_all(db())
    .await
    .map_err(err)?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(server::assemble(row).await.map_err(err)?);
    }
    Ok(out)
}

#[server]
pub async fn create_reservation(input: NewReservation) -> Result<i64, ServerFnError> {
    use crate::backend::{auth, db::{db, now}, notify};
    use crate::models::{hu_range, parse_iso};

    let user = auth::require_user().await?;
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(ServerFnError::new("Adj nevet a foglalásnak."));
    }
    let (Some(from), Some(to)) = (parse_iso(&input.from), parse_iso(&input.to)) else {
        return Err(ServerFnError::new("Érvénytelen dátum."));
    };
    if to < from {
        return Err(ServerFnError::new("A záró nap nem lehet a kezdőnap előtt."));
    }
    if !matches!(input.access.as_str(), "closed" | "open") {
        return Err(ServerFnError::new("Érvénytelen hozzáférés."));
    }

    // Only unreserved days can be requested: overlap with any non-rejected
    // reservation is refused.
    let overlapping: Vec<server::ResRow> = sqlx::query_as(
        "SELECT id, title, day_from, day_to, owner_id, access, note FROM reservations
         WHERE day_from <= ? AND day_to >= ?",
    )
    .bind(&input.to)
    .bind(&input.from)
    .fetch_all(db())
    .await
    .map_err(err)?;
    for row in overlapping {
        let dto = server::assemble(row).await.map_err(err)?;
        if dto.status != "reject" {
            return Err(ServerFnError::new(format!(
                "Ütközés: „{}” ({}).",
                dto.title,
                hu_range(&dto.from, &dto.to)
            )));
        }
    }

    let result = sqlx::query(
        "INSERT INTO reservations (title, day_from, day_to, owner_id, access, note, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&title)
    .bind(&input.from)
    .bind(&input.to)
    .bind(user.id)
    .bind(&input.access)
    .bind(input.note.trim())
    .bind(now())
    .execute(db())
    .await
    .map_err(err)?;
    let res_id = result.last_insert_rowid();

    sqlx::query("INSERT INTO reservation_attendees (reservation_id, user_id) VALUES (?, ?)")
        .bind(res_id)
        .bind(user.id)
        .execute(db())
        .await
        .map_err(err)?;

    // One approval row per active approver; requesting your own reservation
    // counts as your approval if you are an approver yourself.
    let approvers: Vec<(i64,)> =
        sqlx::query_as("SELECT id FROM users WHERE role = 'approver' AND active = 1")
            .fetch_all(db())
            .await
            .map_err(err)?;
    for (approver_id,) in &approvers {
        let own = *approver_id == user.id;
        sqlx::query(
            "INSERT INTO reservation_approvals (reservation_id, approver_id, status, decided_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind(res_id)
        .bind(approver_id)
        .bind(if own { "approved" } else { "pending" })
        .bind(if own { Some(now()) } else { None })
        .execute(db())
        .await
        .map_err(err)?;
    }

    let recipients: Vec<i64> = approvers
        .iter()
        .map(|(id,)| *id)
        .filter(|id| *id != user.id)
        .collect();
    notify::notify(
        &recipients,
        notify::Notice {
            pref_key: "res_request",
            icon: "clock",
            tone: "pending",
            text: format!(
                "{} foglalást kért: „{}” ({}).",
                user.name,
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
#[server]
pub async fn decide_reservation(
    res_id: i64,
    approve: bool,
    comment: String,
) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::{db, now}, notify};

    let approver = auth::require_approver().await?;
    let comment = comment.trim().to_string();
    if !approve && comment.is_empty() {
        return Err(ServerFnError::new("Elutasításhoz indoklás szükséges."));
    }
    let dto = server::fetch(res_id)
        .await
        .map_err(err)?
        .ok_or_else(|| ServerFnError::new("A foglalás nem található."))?;

    sqlx::query(
        "INSERT INTO reservation_approvals (reservation_id, approver_id, status, comment, decided_at)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT (reservation_id, approver_id)
         DO UPDATE SET status = excluded.status, comment = excluded.comment, decided_at = excluded.decided_at",
    )
    .bind(res_id)
    .bind(approver.id)
    .bind(if approve { "approved" } else { "rejected" })
    .bind(&comment)
    .bind(now())
    .execute(db())
    .await
    .map_err(err)?;

    let text = if approve {
        format!("{} jóváhagyta a(z) „{}” foglalást.", approver.name, dto.title)
    } else {
        format!(
            "{} elutasította a(z) „{}” foglalást: „{}”",
            approver.name, dto.title, comment
        )
    };
    server::system_message(
        res_id,
        &if approve {
            format!("{} jóváhagyta a foglalást", approver.name)
        } else {
            format!("{} elutasította a foglalást: „{}”", approver.name, comment)
        },
    )
    .await;
    if dto.owner != approver.id {
        notify::notify(
            &[dto.owner],
            notify::Notice {
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

#[server]
pub async fn set_access(res_id: i64, access: String) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};

    let user = auth::require_user().await?;
    if !matches!(access.as_str(), "closed" | "open") {
        return Err(ServerFnError::new("Érvénytelen hozzáférés."));
    }
    let owner: Option<(i64,)> = sqlx::query_as("SELECT owner_id FROM reservations WHERE id = ?")
        .bind(res_id)
        .fetch_optional(db())
        .await
        .map_err(err)?;
    let Some((owner_id,)) = owner else {
        return Err(ServerFnError::new("A foglalás nem található."));
    };
    if owner_id != user.id {
        return Err(ServerFnError::new("A hozzáférést csak a foglaló módosíthatja."));
    }
    sqlx::query("UPDATE reservations SET access = ? WHERE id = ?")
        .bind(&access)
        .bind(res_id)
        .execute(db())
        .await
        .map_err(err)?;
    server::system_message(
        res_id,
        &if access == "open" {
            format!("{} nyitottá tette a foglalást — bárki csatlakozhat", user.name)
        } else {
            format!("{} zárttá tette a foglalást", user.name)
        },
    )
    .await;
    Ok(())
}

/// Join an open reservation (or leave it again).
#[server]
pub async fn set_attendance(res_id: i64, attend: bool) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db, notify};

    let user = auth::require_user().await?;
    let dto = server::fetch(res_id)
        .await
        .map_err(err)?
        .ok_or_else(|| ServerFnError::new("A foglalás nem található."))?;
    if dto.owner == user.id {
        return Err(ServerFnError::new("A foglaló mindig résztvevő."));
    }
    if dto.access != "open" {
        return Err(ServerFnError::new("Zárt foglaláshoz csak a foglaló adhat hozzá résztvevőt."));
    }
    if dto.status == "reject" {
        return Err(ServerFnError::new("Elutasított foglaláshoz nem lehet csatlakozni."));
    }
    if attend {
        sqlx::query(
            "INSERT OR IGNORE INTO reservation_attendees (reservation_id, user_id) VALUES (?, ?)",
        )
        .bind(res_id)
        .bind(user.id)
        .execute(db())
        .await
        .map_err(err)?;
        server::system_message(res_id, &format!("{} csatlakozott a foglaláshoz", user.name)).await;
        notify::notify(
            &[dto.owner],
            notify::Notice {
                pref_key: "res_join",
                icon: "users",
                tone: "closed",
                text: format!(
                    "{} csatlakozott a(z) „{}” nyitott foglaláshoz.",
                    user.name, dto.title
                ),
                link_kind: Some("reservation"),
                link_id: Some(res_id),
            },
        )
        .await;
    } else {
        sqlx::query("DELETE FROM reservation_attendees WHERE reservation_id = ? AND user_id = ?")
            .bind(res_id)
            .bind(user.id)
            .execute(db())
            .await
            .map_err(err)?;
        server::system_message(res_id, &format!("{} lemondta a részvételt", user.name)).await;
    }
    Ok(())
}

/// Owner manages the attendee list (closed reservations, or corrections).
#[server]
pub async fn set_attendee(res_id: i64, user_id: i64, attend: bool) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};
    use crate::backend::auth::DbUser;

    let user = auth::require_user().await?;
    let dto = server::fetch(res_id)
        .await
        .map_err(err)?
        .ok_or_else(|| ServerFnError::new("A foglalás nem található."))?;
    if dto.owner != user.id {
        return Err(ServerFnError::new("A résztvevőket csak a foglaló kezelheti."));
    }
    if user_id == dto.owner && !attend {
        return Err(ServerFnError::new("A foglaló mindig résztvevő."));
    }
    let target: Option<DbUser> = sqlx::query_as(
        "SELECT id, name, email, color, role, active FROM users WHERE id = ? AND active = 1",
    )
    .bind(user_id)
    .fetch_optional(db())
    .await
    .map_err(err)?;
    let Some(target) = target else {
        return Err(ServerFnError::new("A felhasználó nem található."));
    };
    if attend {
        sqlx::query(
            "INSERT OR IGNORE INTO reservation_attendees (reservation_id, user_id) VALUES (?, ?)",
        )
        .bind(res_id)
        .bind(user_id)
        .execute(db())
        .await
        .map_err(err)?;
        server::system_message(
            res_id,
            &format!("{} hozzáadta {} résztvevőt", user.name, target.name),
        )
        .await;
    } else {
        sqlx::query("DELETE FROM reservation_attendees WHERE reservation_id = ? AND user_id = ?")
            .bind(res_id)
            .bind(user_id)
            .execute(db())
            .await
            .map_err(err)?;
        server::system_message(
            res_id,
            &format!("{} eltávolította {} résztvevőt", user.name, target.name),
        )
        .await;
    }
    Ok(())
}

/// Cancel a reservation (owner, or an approver).
#[server]
pub async fn delete_reservation(res_id: i64) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db, notify};

    let user = auth::require_user().await?;
    let dto = server::fetch(res_id)
        .await
        .map_err(err)?
        .ok_or_else(|| ServerFnError::new("A foglalás nem található."))?;
    if dto.owner != user.id && user.role != "approver" {
        return Err(ServerFnError::new("Csak a foglaló vagy egy engedélyező törölheti."));
    }
    server::system_message(
        res_id,
        &format!("{} törölte a(z) „{}” foglalást", user.name, dto.title),
    )
    .await;
    sqlx::query("DELETE FROM reservations WHERE id = ?")
        .bind(res_id)
        .execute(db())
        .await
        .map_err(err)?;
    let recipients: Vec<i64> = dto
        .attendees
        .iter()
        .copied()
        .filter(|id| *id != user.id)
        .collect();
    notify::notify(
        &recipients,
        notify::Notice {
            pref_key: "res_decision",
            icon: "x",
            tone: "reject",
            text: format!(
                "{} törölte a(z) „{}” foglalást ({}).",
                user.name,
                dto.title,
                server::range_label(&dto)
            ),
            link_kind: None,
            link_id: None,
        },
    )
    .await;
    Ok(())
}

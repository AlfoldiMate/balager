//! In-app notifications and notification preferences.

use dioxus::prelude::*;

use crate::models::{NotifDto, PrefDto};

#[cfg(feature = "server")]
fn err(e: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::new(e.to_string())
}

#[server]
pub async fn list_notifications() -> Result<Vec<NotifDto>, ServerFnError> {
    use crate::backend::{auth, db::db};

    let user = auth::require_user().await?;
    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        icon: String,
        tone: String,
        text: String,
        link_kind: Option<String>,
        link_id: Option<i64>,
        read: i64,
        created_at: i64,
    }
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, icon, tone, text, link_kind, link_id, read, created_at
         FROM notifications WHERE user_id = ? ORDER BY created_at DESC, id DESC LIMIT 30",
    )
    .bind(user.id)
    .fetch_all(db())
    .await
    .map_err(err)?;
    Ok(rows
        .into_iter()
        .map(|r| NotifDto {
            id: r.id,
            icon: r.icon,
            tone: r.tone,
            unread: r.read == 0,
            created_at: r.created_at,
            text: r.text,
            link_kind: r.link_kind,
            link_id: r.link_id,
        })
        .collect())
}

#[server]
pub async fn mark_notification_read(notif_id: i64) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};

    let user = auth::require_user().await?;
    sqlx::query("UPDATE notifications SET read = 1 WHERE id = ? AND user_id = ?")
        .bind(notif_id)
        .bind(user.id)
        .execute(db())
        .await
        .map_err(err)?;
    Ok(())
}

#[server]
pub async fn mark_all_notifications_read() -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};

    let user = auth::require_user().await?;
    sqlx::query("UPDATE notifications SET read = 1 WHERE user_id = ?")
        .bind(user.id)
        .execute(db())
        .await
        .map_err(err)?;
    Ok(())
}

/// Stored preferences merged with the defaults for unset keys.
#[server]
pub async fn get_prefs() -> Result<Vec<PrefDto>, ServerFnError> {
    use crate::backend::{auth, db::db, notify};

    let user = auth::require_user().await?;
    let stored: Vec<(String, i64, i64)> =
        sqlx::query_as("SELECT key, email, push FROM notif_prefs WHERE user_id = ?")
            .bind(user.id)
            .fetch_all(db())
            .await
            .map_err(err)?;
    Ok(notify::PREF_DEFAULTS
        .iter()
        .map(|(key, email_default, push_default)| {
            match stored.iter().find(|(k, _, _)| k == key) {
                Some((_, e, p)) => PrefDto { key: key.to_string(), email: *e != 0, push: *p != 0 },
                None => PrefDto { key: key.to_string(), email: *email_default, push: *push_default },
            }
        })
        .collect())
}

#[server]
pub async fn set_pref(key: String, email: bool, push: bool) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db, notify};

    let user = auth::require_user().await?;
    if !notify::PREF_DEFAULTS.iter().any(|(k, _, _)| *k == key) {
        return Err(ServerFnError::new("Ismeretlen értesítési beállítás."));
    }
    sqlx::query(
        "INSERT INTO notif_prefs (user_id, key, email, push) VALUES (?, ?, ?, ?)
         ON CONFLICT (user_id, key) DO UPDATE SET email = excluded.email, push = excluded.push",
    )
    .bind(user.id)
    .bind(&key)
    .bind(email as i64)
    .bind(push as i64)
    .execute(db())
    .await
    .map_err(err)?;
    Ok(())
}

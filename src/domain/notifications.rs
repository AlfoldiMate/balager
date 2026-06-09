//! In-app notifications and notification preferences.

use super::{invalid, Actor, DomainResult};
use crate::backend::db::db;
use crate::backend::notify;
use crate::models::{NotifDto, PrefDto};

pub async fn list(actor: &Actor) -> DomainResult<Vec<NotifDto>> {
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
         FROM notifications WHERE user_id = $1 ORDER BY created_at DESC, id DESC LIMIT 30",
    )
    .bind(actor.id)
    .fetch_all(db())
    .await?;
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

pub async fn mark_read(actor: &Actor, notif_id: i64) -> DomainResult<()> {
    sqlx::query("UPDATE notifications SET read = 1 WHERE id = $1 AND user_id = $2")
        .bind(notif_id)
        .bind(actor.id)
        .execute(db())
        .await?;
    Ok(())
}

pub async fn mark_all_read(actor: &Actor) -> DomainResult<()> {
    sqlx::query("UPDATE notifications SET read = 1 WHERE user_id = $1")
        .bind(actor.id)
        .execute(db())
        .await?;
    Ok(())
}

/// Stored preferences merged with the defaults for unset keys.
pub async fn get_prefs(actor: &Actor) -> DomainResult<Vec<PrefDto>> {
    let stored: Vec<(String, i64, i64)> =
        sqlx::query_as("SELECT key, email, push FROM notif_prefs WHERE user_id = $1")
            .bind(actor.id)
            .fetch_all(db())
            .await?;
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

pub async fn set_pref(actor: &Actor, key: String, email: bool, push: bool) -> DomainResult<()> {
    if !notify::PREF_DEFAULTS.iter().any(|(k, _, _)| *k == key) {
        return Err(invalid("Ismeretlen értesítési beállítás."));
    }
    sqlx::query(
        "INSERT INTO notif_prefs (user_id, key, email, push) VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, key) DO UPDATE SET email = excluded.email, push = excluded.push",
    )
    .bind(actor.id)
    .bind(&key)
    .bind(email as i64)
    .bind(push as i64)
    .execute(db())
    .await?;
    Ok(())
}

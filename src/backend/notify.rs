//! Notification fan-out: in-app rows always, e-mail when the user's
//! preference allows it and SMTP is configured.

use super::db::{db, now};

/// Default e-mail/push matrix per preference key (matches the design).
pub const PREF_DEFAULTS: &[(&str, bool, bool)] = &[
    ("res_decision", true, true),
    ("res_request", true, true),
    ("res_join", false, true),
    ("task_assigned", true, true),
    ("task_due", false, true),
    ("disc_reply", false, true),
    ("disc_new", false, false),
];

pub fn pref_default(key: &str) -> (bool, bool) {
    PREF_DEFAULTS
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, e, p)| (*e, *p))
        .unwrap_or((false, true))
}

pub struct Notice<'a> {
    pub pref_key: &'a str,
    pub icon: &'a str,
    pub tone: &'a str,
    pub text: String,
    pub link_kind: Option<&'a str>,
    pub link_id: Option<i64>,
}

/// Notify a set of users about an event. Never fails the calling operation.
pub async fn notify(user_ids: &[i64], notice: Notice<'_>) {
    for &uid in user_ids {
        if let Err(e) = sqlx::query(
            "INSERT INTO notifications (user_id, icon, tone, text, link_kind, link_id, read, created_at)
             VALUES (?, ?, ?, ?, ?, ?, 0, ?)",
        )
        .bind(uid)
        .bind(notice.icon)
        .bind(notice.tone)
        .bind(&notice.text)
        .bind(notice.link_kind)
        .bind(notice.link_id)
        .bind(now())
        .execute(db())
        .await
        {
            tracing::warn!("failed to insert notification for user {uid}: {e}");
            continue;
        }
        if email_enabled(uid, notice.pref_key).await {
            send_email(uid, notice.text.clone());
        }
    }
}

async fn email_enabled(user_id: i64, pref_key: &str) -> bool {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT email FROM notif_prefs WHERE user_id = ? AND key = ?")
            .bind(user_id)
            .bind(pref_key)
            .fetch_optional(db())
            .await
            .ok()
            .flatten();
    match row {
        Some((email,)) => email != 0,
        None => pref_default(pref_key).0,
    }
}

struct SmtpConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    from: String,
}

fn smtp_config() -> Option<SmtpConfig> {
    Some(SmtpConfig {
        host: std::env::var("SMTP_HOST").ok()?,
        port: std::env::var("SMTP_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(587),
        username: std::env::var("SMTP_USERNAME").ok()?,
        password: std::env::var("SMTP_PASSWORD").ok()?,
        from: std::env::var("SMTP_FROM").ok()?,
    })
}

/// Fire-and-forget e-mail. Skipped (with a log line) when SMTP is not configured.
fn send_email(user_id: i64, text: String) {
    let Some(cfg) = smtp_config() else {
        tracing::debug!("SMTP not configured; skipping e-mail notification");
        return;
    };
    tokio::spawn(async move {
        let address: Option<(String, String)> =
            sqlx::query_as("SELECT email, name FROM users WHERE id = ? AND active = 1")
                .bind(user_id)
                .fetch_optional(db())
                .await
                .ok()
                .flatten();
        let Some((email, name)) = address else { return };

        let base_url =
            std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into());
        let body = format!(
            "Szia {name}!\n\n{text}\n\nRészletek: {base_url}\n\n— Balager"
        );
        let result = (|| -> Result<lettre::Message, Box<dyn std::error::Error + Send + Sync>> {
            Ok(lettre::Message::builder()
                .from(cfg.from.parse()?)
                .to(email.parse()?)
                .subject("Balager — értesítés")
                .body(body)?)
        })();
        let message = match result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("failed to build notification e-mail: {e}");
                return;
            }
        };
        use lettre::AsyncTransport;
        let transport = match lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(
            &cfg.host,
        ) {
            Ok(builder) => builder
                .port(cfg.port)
                .credentials(lettre::transport::smtp::authentication::Credentials::new(
                    cfg.username,
                    cfg.password,
                ))
                .build(),
            Err(e) => {
                tracing::warn!("invalid SMTP configuration: {e}");
                return;
            }
        };
        if let Err(e) = transport.send(message).await {
            tracing::warn!("failed to send notification e-mail: {e}");
        }
    });
}

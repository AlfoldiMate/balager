//! Password hashing, sessions and the session cookie.

use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use dioxus::fullstack::FullstackContext;
use dioxus::prelude::*;
use rand::RngCore;

use super::db::{db, now};
use crate::models::UserDto;

pub const SESSION_COOKIE: &str = "balager_session";
/// "Long remaining" auth: 180 days.
const SESSION_TTL_SECS: i64 = 180 * 24 * 60 * 60;

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("argon2 hashing failed")
        .to_string()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .map(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false)
}

fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[derive(sqlx::FromRow)]
pub struct DbUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub color: String,
    pub role: String,
    pub active: i64,
}

impl DbUser {
    pub fn dto(&self) -> UserDto {
        UserDto {
            id: self.id,
            name: self.name.clone(),
            email: self.email.clone(),
            color: self.color.clone(),
            approver: self.role == "approver",
            active: self.active != 0,
        }
    }
}

pub async fn create_session(user_id: i64) -> Result<String, sqlx::Error> {
    let token = random_token();
    sqlx::query("INSERT INTO sessions (token, user_id, expires_at) VALUES ($1, $2, $3)")
        .bind(&token)
        .bind(user_id)
        .bind(now() + SESSION_TTL_SECS)
        .execute(db())
        .await?;
    Ok(token)
}

fn secure_attr() -> &'static str {
    // On Vercel (and any HTTPS deployment) mark the cookie Secure; plain
    // localhost development stays on http.
    if std::env::var("VERCEL").is_ok() || std::env::var("BALAGER_SECURE_COOKIES").is_ok() {
        "; Secure"
    } else {
        ""
    }
}

pub fn session_cookie(token: &str) -> String {
    format!(
        "{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={SESSION_TTL_SECS}{}",
        secure_attr()
    )
}

pub fn clear_session_cookie() -> String {
    format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{}", secure_attr())
}

/// Read the session cookie of the current request.
pub async fn request_session_token() -> Option<String> {
    let headers = FullstackContext::extract::<http::HeaderMap, _>().await.ok()?;
    let cookies = headers.get(http::header::COOKIE)?.to_str().ok()?;
    cookies.split(';').find_map(|pair| {
        let (k, v) = pair.trim().split_once('=')?;
        (k == SESSION_COOKIE).then(|| v.to_string())
    })
}

pub fn set_response_cookie(cookie: String) {
    if let Some(ctx) = FullstackContext::current() {
        if let Ok(value) = http::header::HeaderValue::from_str(&cookie) {
            ctx.add_response_header(http::header::SET_COOKIE, value);
        }
    }
}

/// The logged-in user of the current request, if any.
pub async fn current_user() -> Option<DbUser> {
    let token = request_session_token().await?;
    let user: Option<DbUser> = sqlx::query_as(
        "SELECT u.id, u.name, u.email, u.color, u.role, u.active
         FROM sessions s JOIN users u ON u.id = s.user_id
         WHERE s.token = $1 AND s.expires_at > $2 AND u.active = 1",
    )
    .bind(&token)
    .bind(now())
    .fetch_optional(db())
    .await
    .ok()?;
    user
}

/// Guard: any logged-in user.
pub async fn require_user() -> Result<DbUser, ServerFnError> {
    current_user()
        .await
        .ok_or_else(|| ServerFnError::new("Bejelentkezés szükséges."))
}

pub async fn delete_session(token: &str) {
    let _ = sqlx::query("DELETE FROM sessions WHERE token = $1")
        .bind(token)
        .execute(db())
        .await;
}

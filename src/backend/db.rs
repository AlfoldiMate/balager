//! Database pool, migrations and first-run seeding.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;
use std::sync::OnceLock;

static POOL: OnceLock<SqlitePool> = OnceLock::new();

pub fn db() -> &'static SqlitePool {
    POOL.get().expect("database not initialised")
}

pub fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

pub async fn init() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:balager.db".to_string());
    let options = SqliteConnectOptions::from_str(&url)?
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    seed(&pool).await?;
    let _ = POOL.set(pool);
    Ok(())
}

/// On an empty database create the initial approver so the family can log in
/// and create the rest of the accounts from the UI.
async fn seed(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    if user_count > 0 {
        return Ok(());
    }
    let email =
        std::env::var("BALAGER_ADMIN_EMAIL").unwrap_or_else(|_| "admin@balager.hu".to_string());
    let password =
        std::env::var("BALAGER_ADMIN_PASSWORD").unwrap_or_else(|_| "balaton26".to_string());
    let hash = super::auth::hash_password(&password);
    sqlx::query(
        "INSERT INTO users (name, email, password_hash, color, role, active, created_at)
         VALUES (?, ?, ?, '#356b9b', 'approver', 1, ?)",
    )
    .bind("Admin")
    .bind(&email)
    .bind(&hash)
    .bind(now())
    .execute(pool)
    .await?;
    tracing::info!("seeded initial approver account: {email}");
    println!("Balager: created initial approver account: {email} (set BALAGER_ADMIN_PASSWORD to override the default password)");
    Ok(())
}

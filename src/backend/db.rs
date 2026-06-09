//! Database pool (PostgreSQL), migrations and first-run seeding.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::OnceLock;

static POOL: OnceLock<PgPool> = OnceLock::new();

pub fn db() -> &'static PgPool {
    POOL.get().expect("database not initialised")
}

pub fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

pub async fn init() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if POOL.get().is_some() {
        return Ok(());
    }
    let url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is not set (e.g. a Neon/Vercel Postgres connection string)")?;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    seed(&pool).await?;
    let _ = POOL.set(pool);
    Ok(())
}

/// On an empty database create the initial approver so the family can log in
/// and create the rest of the accounts from the UI.
async fn seed(pool: &PgPool) -> Result<(), sqlx::Error> {
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    if user_count > 0 {
        return Ok(());
    }
    let email = std::env::var("BALAGER_ADMIN_EMAIL")
        .unwrap_or_else(|_| "admin@balager.hu".to_string())
        .to_lowercase();
    let password =
        std::env::var("BALAGER_ADMIN_PASSWORD").unwrap_or_else(|_| "balaton26".to_string());
    let hash = super::auth::hash_password(&password);
    sqlx::query(
        "INSERT INTO users (name, email, password_hash, color, role, active, created_at)
         VALUES ($1, $2, $3, '#356b9b', 'approver', 1, $4)",
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

//! Users: administration by approvers, own-profile management.
//! Users cannot register themselves — accounts are created by approvers.

use super::{forbidden, invalid, require_approver, Actor, DomainResult};
use crate::backend::auth::{hash_password, verify_password, DbUser};
use crate::backend::db::{db, now};
use crate::models::UserDto;

pub async fn list_active() -> DomainResult<Vec<UserDto>> {
    let users: Vec<DbUser> = sqlx::query_as(
        "SELECT id, name, email, color, role, active FROM users WHERE active = 1 ORDER BY name",
    )
    .fetch_all(db())
    .await?;
    Ok(users.iter().map(|u| u.dto()).collect())
}

pub async fn list_all(actor: &Actor) -> DomainResult<Vec<UserDto>> {
    require_approver(actor)?;
    let users: Vec<DbUser> = sqlx::query_as(
        "SELECT id, name, email, color, role, active FROM users ORDER BY active DESC, name",
    )
    .fetch_all(db())
    .await?;
    Ok(users.iter().map(|u| u.dto()).collect())
}

pub async fn create(
    actor: &Actor,
    name: String,
    email: String,
    password: String,
    color: String,
    approver: bool,
) -> DomainResult<i64> {
    require_approver(actor)?;
    let name = name.trim().to_string();
    let email = email.trim().to_lowercase();
    if name.is_empty() || !email.contains('@') {
        return Err(invalid("Adj meg érvényes nevet és e-mail címet."));
    }
    if password.chars().count() < 6 {
        return Err(invalid("A jelszó legyen legalább 6 karakter."));
    }
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(db())
        .await?;
    if exists > 0 {
        return Err(invalid("Ezzel az e-mail címmel már létezik felhasználó."));
    }
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO users (name, email, password_hash, color, role, active, created_at)
         VALUES ($1, $2, $3, $4, $5, 1, $6) RETURNING id",
    )
    .bind(&name)
    .bind(&email)
    .bind(hash_password(&password))
    .bind(&color)
    .bind(if approver { "approver" } else { "normal" })
    .bind(now())
    .fetch_one(db())
    .await?;
    Ok(id)
}

pub async fn set_active(actor: &Actor, user_id: i64, active: bool) -> DomainResult<()> {
    require_approver(actor)?;
    if actor.id == user_id && !active {
        return Err(forbidden("Saját fiókodat nem tilthatod le."));
    }
    sqlx::query("UPDATE users SET active = $1 WHERE id = $2")
        .bind(active as i64)
        .bind(user_id)
        .execute(db())
        .await?;
    if !active {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(db())
            .await?;
    }
    Ok(())
}

pub async fn set_role(actor: &Actor, user_id: i64, approver: bool) -> DomainResult<()> {
    require_approver(actor)?;
    if !approver {
        // The house must always keep at least one active approver.
        let approvers: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE role = 'approver' AND active = 1 AND id != $1",
        )
        .bind(user_id)
        .fetch_one(db())
        .await?;
        if approvers == 0 {
            return Err(invalid("Legalább egy engedélyezőnek maradnia kell."));
        }
    }
    sqlx::query("UPDATE users SET role = $1 WHERE id = $2")
        .bind(if approver { "approver" } else { "normal" })
        .bind(user_id)
        .execute(db())
        .await?;
    Ok(())
}

pub async fn reset_password(actor: &Actor, user_id: i64, new_password: String) -> DomainResult<()> {
    require_approver(actor)?;
    if new_password.chars().count() < 6 {
        return Err(invalid("A jelszó legyen legalább 6 karakter."));
    }
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(hash_password(&new_password))
        .bind(user_id)
        .execute(db())
        .await?;
    Ok(())
}

pub async fn update_profile(
    actor: &Actor,
    name: String,
    email: String,
    color: String,
) -> DomainResult<()> {
    let name = name.trim().to_string();
    let email = email.trim().to_lowercase();
    if name.is_empty() || !email.contains('@') {
        return Err(invalid("Adj meg érvényes nevet és e-mail címet."));
    }
    let clash: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1 AND id != $2")
        .bind(&email)
        .bind(actor.id)
        .fetch_one(db())
        .await?;
    if clash > 0 {
        return Err(invalid("Ezzel az e-mail címmel már létezik felhasználó."));
    }
    sqlx::query("UPDATE users SET name = $1, email = $2, color = $3 WHERE id = $4")
        .bind(&name)
        .bind(&email)
        .bind(&color)
        .bind(actor.id)
        .execute(db())
        .await?;
    Ok(())
}

pub async fn change_password(actor: &Actor, current: String, new_password: String) -> DomainResult<()> {
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
        .bind(actor.id)
        .fetch_one(db())
        .await?;
    if !verify_password(&current, &hash) {
        return Err(invalid("A jelenlegi jelszó nem megfelelő."));
    }
    if new_password.chars().count() < 6 {
        return Err(invalid("Az új jelszó legyen legalább 6 karakter."));
    }
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(hash_password(&new_password))
        .bind(actor.id)
        .execute(db())
        .await?;
    Ok(())
}

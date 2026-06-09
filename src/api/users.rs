//! User administration (approvers) and own-profile management.
//! Users cannot register themselves — accounts are created by approvers.

use dioxus::prelude::*;

use crate::models::UserDto;

#[cfg(feature = "server")]
fn err(e: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::new(e.to_string())
}

/// Active users, for pickers/avatars. Any logged-in user.
#[server]
pub async fn list_users() -> Result<Vec<UserDto>, ServerFnError> {
    use crate::backend::{auth, db::db};

    auth::require_user().await?;
    let users: Vec<auth::DbUser> = sqlx::query_as(
        "SELECT id, name, email, color, role, active FROM users WHERE active = 1 ORDER BY name",
    )
    .fetch_all(db())
    .await
    .map_err(err)?;
    Ok(users.iter().map(|u| u.dto()).collect())
}

/// All users including deactivated ones. Approver only.
#[server]
pub async fn admin_list_users() -> Result<Vec<UserDto>, ServerFnError> {
    use crate::backend::{auth, db::db};

    auth::require_approver().await?;
    let users: Vec<auth::DbUser> = sqlx::query_as(
        "SELECT id, name, email, color, role, active FROM users ORDER BY active DESC, name",
    )
    .fetch_all(db())
    .await
    .map_err(err)?;
    Ok(users.iter().map(|u| u.dto()).collect())
}

#[server]
pub async fn create_user(
    name: String,
    email: String,
    password: String,
    color: String,
    approver: bool,
) -> Result<i64, ServerFnError> {
    use crate::backend::{auth, db};

    auth::require_approver().await?;
    let name = name.trim().to_string();
    let email = email.trim().to_string();
    if name.is_empty() || !email.contains('@') {
        return Err(ServerFnError::new("Adj meg érvényes nevet és e-mail címet."));
    }
    if password.chars().count() < 6 {
        return Err(ServerFnError::new("A jelszó legyen legalább 6 karakter."));
    }
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = ?")
        .bind(&email)
        .fetch_one(db::db())
        .await
        .map_err(err)?;
    if exists > 0 {
        return Err(ServerFnError::new("Ezzel az e-mail címmel már létezik felhasználó."));
    }
    let result = sqlx::query(
        "INSERT INTO users (name, email, password_hash, color, role, active, created_at)
         VALUES (?, ?, ?, ?, ?, 1, ?)",
    )
    .bind(&name)
    .bind(&email)
    .bind(auth::hash_password(&password))
    .bind(&color)
    .bind(if approver { "approver" } else { "normal" })
    .bind(db::now())
    .execute(db::db())
    .await
    .map_err(err)?;
    Ok(result.last_insert_rowid())
}

#[server]
pub async fn set_user_active(user_id: i64, active: bool) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};

    let admin = auth::require_approver().await?;
    if admin.id == user_id && !active {
        return Err(ServerFnError::new("Saját fiókodat nem tilthatod le."));
    }
    sqlx::query("UPDATE users SET active = ? WHERE id = ?")
        .bind(active as i64)
        .bind(user_id)
        .execute(db())
        .await
        .map_err(err)?;
    if !active {
        sqlx::query("DELETE FROM sessions WHERE user_id = ?")
            .bind(user_id)
            .execute(db())
            .await
            .map_err(err)?;
    }
    Ok(())
}

#[server]
pub async fn set_user_role(user_id: i64, approver: bool) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};

    auth::require_approver().await?;
    if !approver {
        let approvers: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE role = 'approver' AND active = 1 AND id != ?",
        )
        .bind(user_id)
        .fetch_one(db())
        .await
        .map_err(err)?;
        if approvers == 0 {
            return Err(ServerFnError::new("Legalább egy engedélyezőnek maradnia kell."));
        }
    }
    sqlx::query("UPDATE users SET role = ? WHERE id = ?")
        .bind(if approver { "approver" } else { "normal" })
        .bind(user_id)
        .execute(db())
        .await
        .map_err(err)?;
    Ok(())
}

#[server]
pub async fn admin_reset_password(user_id: i64, new_password: String) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};

    auth::require_approver().await?;
    if new_password.chars().count() < 6 {
        return Err(ServerFnError::new("A jelszó legyen legalább 6 karakter."));
    }
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(auth::hash_password(&new_password))
        .bind(user_id)
        .execute(db())
        .await
        .map_err(err)?;
    Ok(())
}

#[server]
pub async fn update_profile(name: String, email: String, color: String) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};

    let user = auth::require_user().await?;
    let name = name.trim().to_string();
    let email = email.trim().to_string();
    if name.is_empty() || !email.contains('@') {
        return Err(ServerFnError::new("Adj meg érvényes nevet és e-mail címet."));
    }
    let clash: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = ? AND id != ?")
        .bind(&email)
        .bind(user.id)
        .fetch_one(db())
        .await
        .map_err(err)?;
    if clash > 0 {
        return Err(ServerFnError::new("Ezzel az e-mail címmel már létezik felhasználó."));
    }
    sqlx::query("UPDATE users SET name = ?, email = ?, color = ? WHERE id = ?")
        .bind(&name)
        .bind(&email)
        .bind(&color)
        .bind(user.id)
        .execute(db())
        .await
        .map_err(err)?;
    Ok(())
}

#[server]
pub async fn change_password(current: String, new_password: String) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};

    let user = auth::require_user().await?;
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = ?")
        .bind(user.id)
        .fetch_one(db())
        .await
        .map_err(err)?;
    if !auth::verify_password(&current, &hash) {
        return Err(ServerFnError::new("A jelenlegi jelszó nem megfelelő."));
    }
    if new_password.chars().count() < 6 {
        return Err(ServerFnError::new("Az új jelszó legyen legalább 6 karakter."));
    }
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(auth::hash_password(&new_password))
        .bind(user.id)
        .execute(db())
        .await
        .map_err(err)?;
    Ok(())
}

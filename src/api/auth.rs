//! Login / logout / current session.

use dioxus::prelude::*;

use crate::models::UserDto;

#[server]
pub async fn login(email: String, password: String) -> Result<UserDto, ServerFnError> {
    use crate::backend::{auth, db::db};

    let user: Option<auth::DbUser> = sqlx::query_as(
        "SELECT id, name, email, color, role, active FROM users WHERE email = ? AND active = 1",
    )
    .bind(email.trim())
    .fetch_optional(db())
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let Some(user) = user else {
        return Err(ServerFnError::new("Hibás e-mail cím vagy jelszó."));
    };
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = ?")
        .bind(user.id)
        .fetch_one(db())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    if !auth::verify_password(&password, &hash) {
        return Err(ServerFnError::new("Hibás e-mail cím vagy jelszó."));
    }

    let token = auth::create_session(user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    auth::set_response_cookie(auth::session_cookie(&token));
    Ok(user.dto())
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use crate::backend::auth;

    if let Some(token) = auth::request_session_token().await {
        auth::delete_session(&token).await;
    }
    auth::set_response_cookie(auth::clear_session_cookie());
    Ok(())
}

#[server]
pub async fn me() -> Result<Option<UserDto>, ServerFnError> {
    use crate::backend::auth;

    Ok(auth::current_user().await.map(|u| u.dto()))
}

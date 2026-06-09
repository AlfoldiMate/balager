//! User endpoints: authentication + delegation to `domain::users`.

use dioxus::prelude::*;

use crate::models::UserDto;

#[server]
pub async fn list_users() -> Result<Vec<UserDto>, ServerFnError> {
    crate::backend::auth::require_user().await?;
    crate::domain::users::list_active().await.map_err(super::de)
}

#[server]
pub async fn admin_list_users() -> Result<Vec<UserDto>, ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::users::list_all(&actor).await.map_err(super::de)
}

#[server]
pub async fn create_user(
    name: String,
    email: String,
    password: String,
    color: String,
    approver: bool,
) -> Result<i64, ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::users::create(&actor, name, email, password, color, approver)
        .await
        .map_err(super::de)
}

#[server]
pub async fn set_user_active(user_id: i64, active: bool) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::users::set_active(&actor, user_id, active)
        .await
        .map_err(super::de)
}

#[server]
pub async fn set_user_role(user_id: i64, approver: bool) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::users::set_role(&actor, user_id, approver)
        .await
        .map_err(super::de)
}

#[server]
pub async fn admin_reset_password(user_id: i64, new_password: String) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::users::reset_password(&actor, user_id, new_password)
        .await
        .map_err(super::de)
}

#[server]
pub async fn update_profile(name: String, email: String, color: String) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::users::update_profile(&actor, name, email, color)
        .await
        .map_err(super::de)
}

#[server]
pub async fn change_password(current: String, new_password: String) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::users::change_password(&actor, current, new_password)
        .await
        .map_err(super::de)
}

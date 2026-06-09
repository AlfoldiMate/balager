//! Notification endpoints: authentication + delegation to `domain::notifications`.

use dioxus::prelude::*;

use crate::models::{NotifDto, PrefDto};

#[server]
pub async fn list_notifications() -> Result<Vec<NotifDto>, ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::notifications::list(&actor).await.map_err(super::de)
}

#[server]
pub async fn mark_notification_read(notif_id: i64) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::notifications::mark_read(&actor, notif_id)
        .await
        .map_err(super::de)
}

#[server]
pub async fn mark_all_notifications_read() -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::notifications::mark_all_read(&actor)
        .await
        .map_err(super::de)
}

/// Stored preferences merged with the defaults for unset keys.
#[server]
pub async fn get_prefs() -> Result<Vec<PrefDto>, ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::notifications::get_prefs(&actor)
        .await
        .map_err(super::de)
}

#[server]
pub async fn set_pref(key: String, email: bool, push: bool) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::notifications::set_pref(&actor, key, email, push)
        .await
        .map_err(super::de)
}

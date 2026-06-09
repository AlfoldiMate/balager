//! Discussion endpoints: authentication + delegation to `domain::discussions`.

use dioxus::prelude::*;

use crate::models::{ThreadDetailDto, ThreadDto};

#[server]
pub async fn list_threads() -> Result<Vec<ThreadDto>, ServerFnError> {
    crate::backend::auth::require_user().await?;
    crate::domain::discussions::list().await.map_err(super::de)
}

#[server]
pub async fn get_thread(thread_id: i64) -> Result<ThreadDetailDto, ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::discussions::get(&actor, thread_id)
        .await
        .map_err(super::de)
}

/// New general discussion (linked threads are created from the reservation or
/// task side via [`open_or_create_thread`]).
#[server]
pub async fn create_thread(title: String, body: String) -> Result<i64, ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::discussions::create(&actor, title, body)
        .await
        .map_err(super::de)
}

/// The discussion belonging to a reservation or task — created on first use.
#[server]
pub async fn open_or_create_thread(kind: String, link_id: i64) -> Result<i64, ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::discussions::open_or_create(&actor, kind, link_id)
        .await
        .map_err(super::de)
}

#[server]
pub async fn post_message(
    thread_id: i64,
    parent_id: Option<i64>,
    body: String,
) -> Result<i64, ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::discussions::post_message(&actor, thread_id, parent_id, body)
        .await
        .map_err(super::de)
}

#[server]
pub async fn set_pinned(message_id: i64, pinned: bool) -> Result<(), ServerFnError> {
    crate::backend::auth::require_user().await?;
    crate::domain::discussions::set_pinned(message_id, pinned)
        .await
        .map_err(super::de)
}

/// value: -1 (down), 0 (clear), 1 (up)
#[server]
pub async fn vote_message(message_id: i64, value: i64) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::discussions::vote(&actor, message_id, value)
        .await
        .map_err(super::de)
}

/// Only approvers can close (or reopen) a discussion.
#[server]
pub async fn set_thread_closed(thread_id: i64, closed: bool) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::discussions::set_closed(&actor, thread_id, closed)
        .await
        .map_err(super::de)
}

#[server]
pub async fn delete_thread(thread_id: i64) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::discussions::delete(&actor, thread_id)
        .await
        .map_err(super::de)
}

#[server]
pub async fn create_poll(
    thread_id: i64,
    question: String,
    ptype: String,
    mode: String,
    options: Vec<String>,
) -> Result<i64, ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::discussions::create_poll(&actor, thread_id, question, ptype, mode, options)
        .await
        .map_err(super::de)
}

#[server]
pub async fn poll_vote(option_id: i64, on: bool) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::discussions::poll_vote(&actor, option_id, on)
        .await
        .map_err(super::de)
}

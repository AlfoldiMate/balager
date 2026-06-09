//! Task endpoints: authentication + delegation to `domain::tasks`.

use dioxus::prelude::*;

use crate::models::{TaskGroupDto, TaskInput};

#[server]
pub async fn list_task_groups() -> Result<Vec<TaskGroupDto>, ServerFnError> {
    crate::backend::auth::require_user().await?;
    crate::domain::tasks::list().await.map_err(super::de)
}

#[server]
pub async fn create_group(name: String) -> Result<i64, ServerFnError> {
    crate::backend::auth::require_user().await?;
    crate::domain::tasks::create_group(name).await.map_err(super::de)
}

#[server]
pub async fn create_task(input: TaskInput) -> Result<i64, ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::tasks::create(&actor, input).await.map_err(super::de)
}

#[server]
pub async fn update_task(task_id: i64, input: TaskInput) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::tasks::update(&actor, task_id, input)
        .await
        .map_err(super::de)
}

/// Toggle done. Completing a recurring task advances its due date.
#[server]
pub async fn set_task_done(task_id: i64, done: bool) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::tasks::set_done(&actor, task_id, done)
        .await
        .map_err(super::de)
}

#[server]
pub async fn delete_task(task_id: i64) -> Result<(), ServerFnError> {
    crate::backend::auth::require_user().await?;
    crate::domain::tasks::delete(task_id).await.map_err(super::de)
}

#[server]
pub async fn add_subtask(task_id: i64, title: String) -> Result<i64, ServerFnError> {
    crate::backend::auth::require_user().await?;
    crate::domain::tasks::add_subtask(task_id, title)
        .await
        .map_err(super::de)
}

#[server]
pub async fn set_subtask_done(subtask_id: i64, done: bool) -> Result<(), ServerFnError> {
    crate::backend::auth::require_user().await?;
    crate::domain::tasks::set_subtask_done(subtask_id, done)
        .await
        .map_err(super::de)
}

/// Attach an event to a task: creates a linked **open** reservation.
#[server]
pub async fn attach_event(task_id: i64, from: String, to: String) -> Result<i64, ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::tasks::attach_event(&actor, task_id, from, to)
        .await
        .map_err(super::de)
}

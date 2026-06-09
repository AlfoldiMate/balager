//! Tasks: groups, subtasks, recurring advance, attached events.

use dioxus::prelude::*;

use crate::models::{TaskGroupDto, TaskInput};

#[cfg(feature = "server")]
fn err(e: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::new(e.to_string())
}

#[cfg(feature = "server")]
mod server {
    use crate::backend::db::{db, now};

    /// Insert a system message into the thread linked to a task, if any.
    pub async fn system_message(task_id: i64, text: &str) {
        let thread: Option<(i64,)> =
            sqlx::query_as("SELECT id FROM threads WHERE kind = 'task' AND link_id = ?")
                .bind(task_id)
                .fetch_optional(db())
                .await
                .ok()
                .flatten();
        if let Some((thread_id,)) = thread {
            let _ = sqlx::query(
                "INSERT INTO messages (thread_id, author_id, body, system, created_at)
                 VALUES (?, NULL, ?, 1, ?)",
            )
            .bind(thread_id)
            .bind(text)
            .bind(now())
            .execute(db())
            .await;
        }
    }

    pub fn next_due(due: &str, recurring: &str) -> Option<String> {
        let date = crate::models::parse_iso(due)?;
        let next = match recurring {
            "weekly" => date + chrono::Duration::days(7),
            "biweekly" => date + chrono::Duration::days(14),
            "monthly" => date.checked_add_months(chrono::Months::new(1))?,
            "yearly" => date.checked_add_months(chrono::Months::new(12))?,
            _ => return None,
        };
        Some(crate::models::iso(next))
    }
}

#[server]
pub async fn list_task_groups() -> Result<Vec<TaskGroupDto>, ServerFnError> {
    use crate::backend::{auth, db::db};
    use crate::models::{SubTaskDto, TaskDto, TaskEventDto};

    auth::require_user().await?;
    let groups: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, name FROM task_groups ORDER BY position, id")
            .fetch_all(db())
            .await
            .map_err(err)?;

    #[derive(sqlx::FromRow)]
    struct TaskRow {
        id: i64,
        group_id: i64,
        title: String,
        done: i64,
        due: Option<String>,
        assignee_id: Option<i64>,
        recurring: Option<String>,
        reservation_id: Option<i64>,
        res_title: Option<String>,
        thread_id: Option<i64>,
    }
    let tasks: Vec<TaskRow> = sqlx::query_as(
        "SELECT t.id, t.group_id, t.title, t.done, t.due, t.assignee_id, t.recurring,
                t.reservation_id, r.title AS res_title, th.id AS thread_id
         FROM tasks t
         LEFT JOIN reservations r ON r.id = t.reservation_id
         LEFT JOIN threads th ON th.kind = 'task' AND th.link_id = t.id
         ORDER BY t.done, t.due IS NULL, t.due, t.id",
    )
    .fetch_all(db())
    .await
    .map_err(err)?;
    let subs: Vec<(i64, i64, String, i64)> =
        sqlx::query_as("SELECT id, task_id, title, done FROM subtasks ORDER BY id")
            .fetch_all(db())
            .await
            .map_err(err)?;

    let mut out: Vec<TaskGroupDto> = groups
        .into_iter()
        .map(|(id, name)| TaskGroupDto { id, name, tasks: Vec::new() })
        .collect();
    for row in tasks {
        let dto = TaskDto {
            id: row.id,
            group_id: row.group_id,
            title: row.title,
            done: row.done != 0,
            due: row.due,
            assignee: row.assignee_id,
            recurring: row.recurring,
            event: row.reservation_id.map(|res_id| TaskEventDto {
                res_id,
                label: row.res_title.unwrap_or_default(),
            }),
            subs: subs
                .iter()
                .filter(|(_, task_id, _, _)| *task_id == row.id)
                .map(|(id, _, title, done)| SubTaskDto {
                    id: *id,
                    title: title.clone(),
                    done: *done != 0,
                })
                .collect(),
            thread_id: row.thread_id,
        };
        if let Some(group) = out.iter_mut().find(|g| g.id == dto.group_id) {
            group.tasks.push(dto);
        }
    }
    Ok(out)
}

#[server]
pub async fn create_group(name: String) -> Result<i64, ServerFnError> {
    use crate::backend::{auth, db::db};

    auth::require_user().await?;
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("Adj nevet a csoportnak."));
    }
    let result = sqlx::query("INSERT INTO task_groups (name) VALUES (?)")
        .bind(&name)
        .execute(db())
        .await
        .map_err(err)?;
    Ok(result.last_insert_rowid())
}

#[server]
pub async fn create_task(input: TaskInput) -> Result<i64, ServerFnError> {
    use crate::backend::{auth, db::{db, now}, notify};

    let user = auth::require_user().await?;
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(ServerFnError::new("Adj nevet a feladatnak."));
    }
    let result = sqlx::query(
        "INSERT INTO tasks (group_id, title, done, due, assignee_id, recurring, created_at)
         VALUES (?, ?, 0, ?, ?, ?, ?)",
    )
    .bind(input.group_id)
    .bind(&title)
    .bind(&input.due)
    .bind(input.assignee)
    .bind(&input.recurring)
    .bind(now())
    .execute(db())
    .await
    .map_err(err)?;
    let task_id = result.last_insert_rowid();
    if let Some(assignee) = input.assignee {
        if assignee != user.id {
            notify::notify(
                &[assignee],
                notify::Notice {
                    pref_key: "task_assigned",
                    icon: "tasks",
                    tone: "",
                    text: format!("{} feladatot rendelt hozzád: „{}”.", user.name, title),
                    link_kind: Some("task"),
                    link_id: Some(task_id),
                },
            )
            .await;
        }
    }
    Ok(task_id)
}

#[server]
pub async fn update_task(task_id: i64, input: TaskInput) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db, notify};

    let user = auth::require_user().await?;
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(ServerFnError::new("Adj nevet a feladatnak."));
    }
    let previous: Option<(Option<i64>,)> =
        sqlx::query_as("SELECT assignee_id FROM tasks WHERE id = ?")
            .bind(task_id)
            .fetch_optional(db())
            .await
            .map_err(err)?;
    let Some((prev_assignee,)) = previous else {
        return Err(ServerFnError::new("A feladat nem található."));
    };
    sqlx::query(
        "UPDATE tasks SET group_id = ?, title = ?, due = ?, assignee_id = ?, recurring = ?
         WHERE id = ?",
    )
    .bind(input.group_id)
    .bind(&title)
    .bind(&input.due)
    .bind(input.assignee)
    .bind(&input.recurring)
    .bind(task_id)
    .execute(db())
    .await
    .map_err(err)?;
    if let Some(assignee) = input.assignee {
        if Some(assignee) != prev_assignee && assignee != user.id {
            notify::notify(
                &[assignee],
                notify::Notice {
                    pref_key: "task_assigned",
                    icon: "tasks",
                    tone: "",
                    text: format!("{} feladatot rendelt hozzád: „{}”.", user.name, title),
                    link_kind: Some("task"),
                    link_id: Some(task_id),
                },
            )
            .await;
        }
    }
    Ok(())
}

/// Toggle done. Completing a recurring task advances its due date instead of
/// staying done.
#[server]
pub async fn set_task_done(task_id: i64, done: bool) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};
    use crate::models::hu_date;

    let user = auth::require_user().await?;
    let row: Option<(String, Option<String>, Option<String>)> =
        sqlx::query_as("SELECT title, due, recurring FROM tasks WHERE id = ?")
            .bind(task_id)
            .fetch_optional(db())
            .await
            .map_err(err)?;
    let Some((title, due, recurring)) = row else {
        return Err(ServerFnError::new("A feladat nem található."));
    };

    if done {
        if let (Some(due), Some(recurring)) = (&due, &recurring) {
            if let Some(next) = server::next_due(due, recurring) {
                sqlx::query("UPDATE tasks SET due = ?, done = 0 WHERE id = ?")
                    .bind(&next)
                    .bind(task_id)
                    .execute(db())
                    .await
                    .map_err(err)?;
                sqlx::query("UPDATE subtasks SET done = 0 WHERE task_id = ?")
                    .bind(task_id)
                    .execute(db())
                    .await
                    .map_err(err)?;
                server::system_message(
                    task_id,
                    &format!(
                        "{} elvégezte a(z) „{}” feladatot — következő alkalom: {}",
                        user.name,
                        title,
                        hu_date(&next)
                    ),
                )
                .await;
                return Ok(());
            }
        }
    }
    sqlx::query("UPDATE tasks SET done = ? WHERE id = ?")
        .bind(done as i64)
        .bind(task_id)
        .execute(db())
        .await
        .map_err(err)?;
    server::system_message(
        task_id,
        &if done {
            format!("{} késznek jelölte a(z) „{}” feladatot", user.name, title)
        } else {
            format!("{} újranyitotta a(z) „{}” feladatot", user.name, title)
        },
    )
    .await;
    Ok(())
}

#[server]
pub async fn delete_task(task_id: i64) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};

    auth::require_user().await?;
    sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(task_id)
        .execute(db())
        .await
        .map_err(err)?;
    Ok(())
}

#[server]
pub async fn add_subtask(task_id: i64, title: String) -> Result<i64, ServerFnError> {
    use crate::backend::{auth, db::db};

    auth::require_user().await?;
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(ServerFnError::new("Adj nevet az alfeladatnak."));
    }
    let result = sqlx::query("INSERT INTO subtasks (task_id, title, done) VALUES (?, ?, 0)")
        .bind(task_id)
        .bind(&title)
        .execute(db())
        .await
        .map_err(err)?;
    Ok(result.last_insert_rowid())
}

#[server]
pub async fn set_subtask_done(subtask_id: i64, done: bool) -> Result<(), ServerFnError> {
    use crate::backend::{auth, db::db};

    auth::require_user().await?;
    sqlx::query("UPDATE subtasks SET done = ? WHERE id = ?")
        .bind(done as i64)
        .bind(subtask_id)
        .execute(db())
        .await
        .map_err(err)?;
    Ok(())
}

/// Attach an event to a task: creates an **open** reservation through the
/// normal approval flow and links it to the task.
#[server]
pub async fn attach_event(task_id: i64, from: String, to: String) -> Result<i64, ServerFnError> {
    use crate::backend::{auth, db::db};
    use crate::models::NewReservation;

    auth::require_user().await?;
    let row: Option<(String, Option<i64>)> =
        sqlx::query_as("SELECT title, reservation_id FROM tasks WHERE id = ?")
            .bind(task_id)
            .fetch_optional(db())
            .await
            .map_err(err)?;
    let Some((title, existing)) = row else {
        return Err(ServerFnError::new("A feladat nem található."));
    };
    if existing.is_some() {
        return Err(ServerFnError::new("Ehhez a feladathoz már tartozik esemény."));
    }
    let res_id = super::reservations::create_reservation(NewReservation {
        title,
        from,
        to,
        access: "open".into(),
        note: "Feladathoz kapcsolt esemény.".into(),
    })
    .await?;
    sqlx::query("UPDATE tasks SET reservation_id = ? WHERE id = ?")
        .bind(res_id)
        .bind(task_id)
        .execute(db())
        .await
        .map_err(err)?;
    server::system_message(task_id, "Esemény kapcsolva: nyitott foglalás létrehozva").await;
    Ok(res_id)
}

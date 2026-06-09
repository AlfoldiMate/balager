//! Tasks: groups, subtasks, recurring advance, attached events.

use super::{invalid, not_found, Actor, DomainResult};
use crate::backend::db::{db, now};
use crate::backend::notify::{self, Notice};
use crate::models::{hu_date, NewReservation, SubTaskDto, TaskDto, TaskEventDto, TaskGroupDto, TaskInput};

/// Insert a system message into the thread linked to a task, if any.
async fn system_message(task_id: i64, text: &str) {
    let thread: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM threads WHERE kind = 'task' AND link_id = $1")
            .bind(task_id)
            .fetch_optional(db())
            .await
            .ok()
            .flatten();
    if let Some((thread_id,)) = thread {
        let _ = sqlx::query(
            "INSERT INTO messages (thread_id, author_id, body, system, created_at)
             VALUES ($1, NULL, $2, 1, $3)",
        )
        .bind(thread_id)
        .bind(text)
        .bind(now())
        .execute(db())
        .await;
    }
}

fn next_due(due: &str, recurring: &str) -> Option<String> {
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

pub async fn list() -> DomainResult<Vec<TaskGroupDto>> {
    let groups: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, name FROM task_groups ORDER BY position, id")
            .fetch_all(db())
            .await?;

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
    .await?;
    let subs: Vec<(i64, i64, String, i64)> =
        sqlx::query_as("SELECT id, task_id, title, done FROM subtasks ORDER BY id")
            .fetch_all(db())
            .await?;

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

pub async fn create_group(name: String) -> DomainResult<i64> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(invalid("Adj nevet a csoportnak."));
    }
    let id: i64 = sqlx::query_scalar("INSERT INTO task_groups (name) VALUES ($1) RETURNING id")
        .bind(&name)
        .fetch_one(db())
        .await?;
    Ok(id)
}

pub async fn create(actor: &Actor, input: TaskInput) -> DomainResult<i64> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(invalid("Adj nevet a feladatnak."));
    }
    let task_id: i64 = sqlx::query_scalar(
        "INSERT INTO tasks (group_id, title, done, due, assignee_id, recurring, created_at)
         VALUES ($1, $2, 0, $3, $4, $5, $6) RETURNING id",
    )
    .bind(input.group_id)
    .bind(&title)
    .bind(&input.due)
    .bind(input.assignee)
    .bind(&input.recurring)
    .bind(now())
    .fetch_one(db())
    .await?;
    notify_assignment(actor, input.assignee, None, task_id, &title).await;
    Ok(task_id)
}

pub async fn update(actor: &Actor, task_id: i64, input: TaskInput) -> DomainResult<()> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(invalid("Adj nevet a feladatnak."));
    }
    let previous: Option<(Option<i64>,)> =
        sqlx::query_as("SELECT assignee_id FROM tasks WHERE id = $1")
            .bind(task_id)
            .fetch_optional(db())
            .await?;
    let Some((prev_assignee,)) = previous else {
        return Err(not_found("A feladat nem található."));
    };
    sqlx::query(
        "UPDATE tasks SET group_id = $1, title = $2, due = $3, assignee_id = $4, recurring = $5
         WHERE id = $6",
    )
    .bind(input.group_id)
    .bind(&title)
    .bind(&input.due)
    .bind(input.assignee)
    .bind(&input.recurring)
    .bind(task_id)
    .execute(db())
    .await?;
    notify_assignment(actor, input.assignee, prev_assignee, task_id, &title).await;
    Ok(())
}

async fn notify_assignment(
    actor: &Actor,
    assignee: Option<i64>,
    previous: Option<i64>,
    task_id: i64,
    title: &str,
) {
    if let Some(assignee) = assignee {
        if Some(assignee) != previous && assignee != actor.id {
            notify::notify(
                &[assignee],
                Notice {
                    pref_key: "task_assigned",
                    icon: "tasks",
                    tone: "",
                    text: format!("{} feladatot rendelt hozzád: „{}”.", actor.name, title),
                    link_kind: Some("task"),
                    link_id: Some(task_id),
                },
            )
            .await;
        }
    }
}

/// Toggle done. Completing a recurring task advances its due date instead of
/// staying done.
pub async fn set_done(actor: &Actor, task_id: i64, done: bool) -> DomainResult<()> {
    let row: Option<(String, Option<String>, Option<String>)> =
        sqlx::query_as("SELECT title, due, recurring FROM tasks WHERE id = $1")
            .bind(task_id)
            .fetch_optional(db())
            .await?;
    let Some((title, due, recurring)) = row else {
        return Err(not_found("A feladat nem található."));
    };

    if done {
        if let (Some(due), Some(recurring)) = (&due, &recurring) {
            if let Some(next) = next_due(due, recurring) {
                sqlx::query("UPDATE tasks SET due = $1, done = 0 WHERE id = $2")
                    .bind(&next)
                    .bind(task_id)
                    .execute(db())
                    .await?;
                sqlx::query("UPDATE subtasks SET done = 0 WHERE task_id = $1")
                    .bind(task_id)
                    .execute(db())
                    .await?;
                system_message(
                    task_id,
                    &format!(
                        "{} elvégezte a(z) „{}” feladatot — következő alkalom: {}",
                        actor.name,
                        title,
                        hu_date(&next)
                    ),
                )
                .await;
                return Ok(());
            }
        }
    }
    sqlx::query("UPDATE tasks SET done = $1 WHERE id = $2")
        .bind(done as i64)
        .bind(task_id)
        .execute(db())
        .await?;
    system_message(
        task_id,
        &if done {
            format!("{} késznek jelölte a(z) „{}” feladatot", actor.name, title)
        } else {
            format!("{} újranyitotta a(z) „{}” feladatot", actor.name, title)
        },
    )
    .await;
    Ok(())
}

pub async fn delete(task_id: i64) -> DomainResult<()> {
    sqlx::query("DELETE FROM tasks WHERE id = $1")
        .bind(task_id)
        .execute(db())
        .await?;
    Ok(())
}

pub async fn add_subtask(task_id: i64, title: String) -> DomainResult<i64> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(invalid("Adj nevet az alfeladatnak."));
    }
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO subtasks (task_id, title, done) VALUES ($1, $2, 0) RETURNING id",
    )
    .bind(task_id)
    .bind(&title)
    .fetch_one(db())
    .await?;
    Ok(id)
}

pub async fn set_subtask_done(subtask_id: i64, done: bool) -> DomainResult<()> {
    sqlx::query("UPDATE subtasks SET done = $1 WHERE id = $2")
        .bind(done as i64)
        .bind(subtask_id)
        .execute(db())
        .await?;
    Ok(())
}

/// Attach an event to a task: creates an **open** reservation through the
/// normal approval flow and links it to the task.
pub async fn attach_event(actor: &Actor, task_id: i64, from: String, to: String) -> DomainResult<i64> {
    let row: Option<(String, Option<i64>)> =
        sqlx::query_as("SELECT title, reservation_id FROM tasks WHERE id = $1")
            .bind(task_id)
            .fetch_optional(db())
            .await?;
    let Some((title, existing)) = row else {
        return Err(not_found("A feladat nem található."));
    };
    if existing.is_some() {
        return Err(invalid("Ehhez a feladathoz már tartozik esemény."));
    }
    let res_id = super::reservations::create(
        actor,
        NewReservation {
            title,
            from,
            to,
            access: "open".into(),
            note: "Feladathoz kapcsolt esemény.".into(),
        },
    )
    .await?;
    sqlx::query("UPDATE tasks SET reservation_id = $1 WHERE id = $2")
        .bind(res_id)
        .bind(task_id)
        .execute(db())
        .await?;
    system_message(task_id, "Esemény kapcsolva: nyitott foglalás létrehozva").await;
    Ok(res_id)
}

//! Tasks: grouped tasks, subtasks, recurring, attached events, editor panel.

use dioxus::prelude::*;

use crate::common::Avatar;
use crate::data::{self, hu_date, user, SubTask, Task, TaskGroup as TaskGroupData, ME, TASK_GROUPS};
use crate::icons::Icon;
use crate::state::{AppState, TaskPanel};

#[component]
pub fn TasksHeaderLeft() -> Element {
    let state = use_context::<AppState>();
    let any_open = state.task_open.read().values().any(|v| *v);
    rsx! {
        button {
            class: "bg-btn",
            title: "Új feladat",
            style: "width: 44px; padding: 0; justify-content: center;",
            onclick: move |_| {
                let mut p = state.task_panel;
                p.set(Some(TaskPanel::New));
            },
            Icon { name: "plus", size: 18.0 }
        }
        button { class: "bg-btn ghost", title: "Új csoport", style: "width: 44px; padding: 0; justify-content: center;",
            Icon { name: "folder", size: 17.0 }
        }
        button {
            class: "bg-iconbtn",
            title: if any_open { "Összes csoport összecsukása" } else { "Összes csoport kinyitása" },
            onclick: move |_| {
                let mut open = state.task_open;
                let target = !open.read().values().any(|v| *v);
                open.set(TASK_GROUPS.iter().map(|g| (g.id, target)).collect());
            },
            Icon { name: if any_open { "collapseall" } else { "expandall" }, size: 18.0, stroke: 2.0 }
        }
    }
}

#[component]
pub fn TasksHeaderRight() -> Element {
    let state = use_context::<AppState>();
    let filter = (state.task_filter)();
    let total: usize = TASK_GROUPS.iter().map(|g| g.tasks.len()).sum();
    let open: usize = TASK_GROUPS.iter().map(|g| g.tasks.iter().filter(|t| !t.done).count()).sum();
    rsx! {
        div { class: "hdr-tabs",
            button {
                class: if filter == "all" { "on" } else { "" },
                onclick: move |_| {
                    let mut f = state.task_filter;
                    f.set("all");
                },
                "Mind "
                span { class: "c", "{total}" }
            }
            button {
                class: if filter == "active" { "on" } else { "" },
                onclick: move |_| {
                    let mut f = state.task_filter;
                    f.set("active");
                },
                "Aktív "
                span { class: "c", "{open}" }
            }
            button {
                class: if filter == "done" { "on" } else { "" },
                onclick: move |_| {
                    let mut f = state.task_filter;
                    f.set("done");
                },
                "Kész "
                span { class: "c", "{total - open}" }
            }
        }
    }
}

#[component]
fn SubRow(sub: &'static SubTask) -> Element {
    let mut done = use_signal(|| sub.done);
    let is_done = done();
    rsx! {
        div { class: if is_done { "tk-subrow done" } else { "tk-subrow" },
            button {
                class: if is_done { "tk-check done" } else { "tk-check" },
                onclick: move |_| {
                    let v = done();
                    done.set(!v);
                },
                if is_done {
                    Icon { name: "checkmini", size: 11.0, stroke: 2.6 }
                }
            }
            span { class: "tx", style: "flex: 1;", "{sub.title}" }
        }
    }
}

#[component]
fn TaskMenu(onclose: EventHandler<()>) -> Element {
    rsx! {
        div { class: "ds-menu-scrim", onclick: move |_| onclose.call(()) }
        div { class: "ds-menu", onclick: move |e| e.stop_propagation(),
            button { class: "ds-menu-item", onclick: move |_| onclose.call(()),
                Icon { name: "check", size: 16.0 }
                " Késznek jelöl"
            }
            button { class: "ds-menu-item", onclick: move |_| onclose.call(()),
                Icon { name: "users", size: 16.0 }
                " Áthelyezés máshoz"
            }
            button { class: "ds-menu-item", onclick: move |_| onclose.call(()),
                Icon { name: "repeat", size: 16.0 }
                " Ismétlődés beállítása"
            }
            button { class: "ds-menu-item", onclick: move |_| onclose.call(()),
                Icon { name: "folder", size: 16.0 }
                " Áthelyezés csoportba"
            }
            div { class: "ds-menu-sep" }
            button { class: "ds-menu-item danger", onclick: move |_| onclose.call(()),
                Icon { name: "x", size: 16.0 }
                " Törlés"
            }
        }
    }
}

#[component]
fn TaskItem(task: &'static Task) -> Element {
    let state = use_context::<AppState>();
    let mut done = use_signal(|| task.done);
    let mut open = use_signal(|| false);
    let mut menu = use_signal(|| false);
    let is_done = done();
    let is_open = open();
    let sub_done = task.subs.iter().filter(|s| s.done).count();
    let has_subs = !task.subs.is_empty();
    let assignee = user(task.assignee);

    rsx! {
        div { class: if is_done { "tk-item done" } else { "tk-item" },
            div {
                class: "tk-row tk-clickable",
                onclick: move |_| {
                    let mut p = state.task_panel;
                    p.set(Some(TaskPanel::Edit(task)));
                },
                button {
                    class: if is_done { "tk-check done" } else { "tk-check" },
                    onclick: move |e| {
                        e.stop_propagation();
                        let v = done();
                        done.set(!v);
                    },
                    if is_done {
                        Icon { name: "check", size: 14.0, stroke: 2.6 }
                    }
                }
                div { class: "tk-main",
                    div { class: "tk-title",
                        span { class: "txt", "{task.title}" }
                        if !task.recurring.is_empty() {
                            span { class: "bg-chip reed", style: "height: 22px;",
                                Icon { name: "repeat", size: 13.0 }
                                " {task.recurring}"
                            }
                        }
                    }
                    div { class: "tk-meta",
                        span { class: "m",
                            Avatar { id: "{task.assignee}", size: "sm" }
                            " {assignee.name}"
                        }
                        if !task.due.is_empty() {
                            span { class: "m",
                                Icon { name: "clock" }
                                " {hu_date(task.due)}"
                            }
                        }
                        if has_subs {
                            button {
                                class: "m",
                                style: "cursor: pointer;",
                                onclick: move |e| {
                                    e.stop_propagation();
                                    let v = open();
                                    open.set(!v);
                                },
                                Icon { name: "list" }
                                " {sub_done}/{task.subs.len()} alfeladat "
                                Icon {
                                    name: "chevdown",
                                    size: 13.0,
                                    style: if is_open { "transform: rotate(180deg); transition: transform .18s;" } else { "transition: transform .18s;" },
                                }
                            }
                        }
                        button {
                            class: "m tk-discuss",
                            onclick: move |e| {
                                e.stop_propagation();
                                state.go_discuss(task.id);
                            },
                            Icon { name: "chat" }
                            " Beszélgetés"
                        }
                    }
                }
                if let Some(ev) = &task.event {
                    {
                        let res_id = ev.res_id;
                        rsx! {
                            button {
                                class: "tk-eventchip",
                                title: "Eseményhez kötve: {ev.label}",
                                onclick: move |e| {
                                    e.stop_propagation();
                                    state.open_event(res_id);
                                },
                                Icon { name: "calendar", size: 15.0, stroke: 2.2 }
                            }
                        }
                    }
                }
                div { class: "tk-menuwrap", onclick: move |e| e.stop_propagation(),
                    button {
                        class: if menu() { "bg-iconbtn tk-dots on" } else { "bg-iconbtn tk-dots" },
                        onclick: move |_| {
                            let v = menu();
                            menu.set(!v);
                        },
                        Icon { name: "dots", size: 17.0 }
                    }
                    if menu() {
                        TaskMenu { onclose: move |_| menu.set(false) }
                    }
                }
            }
            if has_subs && is_open {
                div { class: "tk-sub",
                    for s in task.subs {
                        SubRow { key: "{s.id}", sub: s }
                    }
                }
            }
        }
    }
}

#[component]
fn TaskGroup(group: &'static TaskGroupData, tasks: Vec<&'static Task>, open: bool) -> Element {
    let state = use_context::<AppState>();
    let open_count = group.tasks.iter().filter(|t| !t.done).count();
    rsx! {
        div { class: if open { "tk-group" } else { "tk-group closed" },
            button {
                class: "tk-ghead",
                onclick: move |_| {
                    let mut m = state.task_open;
                    let v = m.read().get(group.id).copied().unwrap_or(true);
                    m.write().insert(group.id, !v);
                },
                Icon { name: "chevdown", size: 16.0, stroke: 2.2, class: "gchev" }
                Icon { name: "folder", size: 17.0, stroke: 2.0, style: "color: var(--water);" }
                span { class: "gname", "{group.name}" }
                span { class: "gcount", "{open_count}/{group.tasks.len()}" }
                span { class: "gline" }
            }
            if open {
                div { class: "tk-list",
                    for t in tasks {
                        TaskItem { key: "{t.id}", task: t }
                    }
                }
            }
        }
    }
}

#[component]
fn TaskEditorPanel(panel: TaskPanel) -> Element {
    let state = use_context::<AppState>();
    let device_mobile = (state.is_mobile)();
    let task: Option<&'static Task> = match panel {
        TaskPanel::New => None,
        TaskPanel::Edit(t) => Some(t),
    };
    let is_new = task.is_none();
    let done = task.map(|t| t.done).unwrap_or(false);
    let mut recur = use_signal(|| task.map(|t| t.recurring).unwrap_or(""));
    let group = task.and_then(|t| {
        TASK_GROUPS.iter().find(|g| g.tasks.iter().any(|x| x.id == t.id))
    });
    let group_name = group.map(|g| g.name).unwrap_or("Új feladat");
    let assignee = task.map(|t| t.assignee).unwrap_or(ME);
    let title = task.map(|t| t.title).unwrap_or("");
    let due = task.map(|t| t.due).unwrap_or("");
    let close = move |_| {
        let mut p = state.task_panel;
        p.set(None);
    };

    rsx! {
        aside { class: "bg-panel",
            if device_mobile {
                div { class: "sheet-grab" }
            }
            div { class: "ph",
                div { style: "flex: 1;",
                    div { style: "margin-bottom: 7px; display: flex; gap: 7px;",
                        span { class: "bg-chip",
                            Icon { name: "folder", size: 13.0 }
                            " {group_name}"
                        }
                        if done {
                            span { class: "bg-chip open", style: "height: 26px;",
                                Icon { name: "check", size: 13.0 }
                                " Kész"
                            }
                        }
                    }
                    h3 { if is_new { "Új feladat" } else { "Feladat szerkesztése" } }
                }
                button { class: "bg-iconbtn", onclick: close,
                    Icon { name: "x", size: 18.0 }
                }
            }
            div { class: "pbody",
                div { class: "bg-field",
                    label { "Megnevezés" }
                    input { class: "bg-input", initial_value: "{title}", placeholder: "Mi a teendő?" }
                }
                div { class: "set-pwgrid",
                    div { class: "bg-field",
                        label { "Felelős" }
                        div { class: "bg-input", style: "display: flex; align-items: center; gap: 9px;",
                            Avatar { id: "{assignee}", size: "sm" }
                            " {user(assignee).name}"
                            Icon { name: "chevdown", size: 15.0, style: "margin-left: auto; color: var(--ink-3);" }
                        }
                    }
                    div { class: "bg-field",
                        label { "Határidő" }
                        div { class: "bg-input", style: "display: flex; align-items: center; gap: 9px;",
                            Icon { name: "clock", size: 15.0, style: "color: var(--ink-3);" }
                            if due.is_empty() { "Nincs" } else { "{hu_date(due)}" }
                        }
                    }
                }
                div { class: "bg-field",
                    label { "Ismétlődés" }
                    div { class: "bg-seg",
                        for r in ["", "Hetente", "Kéthetente", "Havonta"] {
                            button {
                                key: "{r}",
                                class: if recur() == r { "on" } else { "" },
                                onclick: move |_| recur.set(r),
                                if r.is_empty() { "Egyszeri" } else { "{r}" }
                            }
                        }
                    }
                }
                if let Some(t) = task {
                    if !t.subs.is_empty() {
                        div { class: "bg-field",
                            label { "Alfeladatok · {t.subs.iter().filter(|s| s.done).count()}/{t.subs.len()}" }
                            div { class: "tk-paneledit-subs",
                                for s in t.subs {
                                    SubRow { key: "{s.id}", sub: s }
                                }
                            }
                            button { class: "bg-btn ghost sm", style: "margin-top: 9px;",
                                Icon { name: "plus", size: 14.0 }
                                " Alfeladat"
                            }
                        }
                    }
                    if let Some(ev) = &t.event {
                        {
                            let res_id = ev.res_id;
                            rsx! {
                                div { class: "bg-field",
                                    label { "Kapcsolt esemény" }
                                    button {
                                        class: "tk-event",
                                        style: "margin: 0; width: 100%;",
                                        onclick: move |_| state.open_event(res_id),
                                        Icon { name: "calendar", size: 15.0, stroke: 2.2 }
                                        " "
                                        b { "{ev.label}" }
                                        " · nyitott foglalás"
                                        Icon { name: "chevright", size: 14.0, style: "margin-left: auto;" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "pfoot",
                button {
                    class: "bg-btn ghost",
                    style: "flex: 1; justify-content: center;",
                    onclick: move |_| {
                        if let Some(t) = task {
                            state.go_discuss(t.id);
                        }
                    },
                    Icon { name: "chat", size: 16.0 }
                    " Beszélgetés"
                }
                button { class: "bg-btn", style: "flex: 1; justify-content: center;", onclick: close,
                    Icon { name: "check", size: 16.0 }
                    " Mentés"
                }
            }
        }
    }
}

#[component]
pub fn TasksTool() -> Element {
    let state = use_context::<AppState>();
    let device_mobile = (state.is_mobile)();
    let filter = (state.task_filter)();
    let open_map = state.task_open.read().clone();

    let match_filter = |t: &Task| match filter {
        "active" => !t.done,
        "done" => t.done,
        _ => true,
    };
    let groups: Vec<(&'static data::TaskGroup, Vec<&'static Task>)> = TASK_GROUPS
        .iter()
        .map(|g| (g, g.tasks.iter().filter(|t| match_filter(t)).collect::<Vec<_>>()))
        .filter(|(_, ts)| !ts.is_empty())
        .collect();

    rsx! {
        div { class: "bg-content bg-fade",
            div { class: "tk-wrap",
                if device_mobile {
                    div { class: "tool-toolbar",
                        div { class: "tt-left", TasksHeaderLeft {} }
                        div { class: "tt-right", TasksHeaderRight {} }
                    }
                }
                div { class: "tk-groups",
                    if groups.is_empty() {
                        div { class: "tk-empty",
                            Icon { name: "check", size: 26.0, stroke: 2.0 }
                            " Nincs ilyen feladat."
                        }
                    } else {
                        for (g, tasks) in groups {
                            TaskGroup {
                                key: "{g.id}",
                                group: g,
                                tasks,
                                open: open_map.get(g.id).copied().unwrap_or(true),
                            }
                        }
                    }
                }
            }
        }
        if let Some(p) = (state.task_panel)() {
            TaskEditorPanel { panel: p }
        }
    }
}

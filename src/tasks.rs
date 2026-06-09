//! Tasks: grouped tasks, subtasks, recurring, attached events, editor panel.

use dioxus::prelude::*;

use crate::api;
use crate::common::{Avatar, ErrorNote};
use crate::icons::Icon;
use crate::login::clean_err;
use crate::models::{hu_date, recurring_label, SubTaskDto, TaskDto, TaskInput, RECURRING_OPTIONS};
use crate::state::{AppState, TaskPanel};

#[component]
pub fn TasksHeaderLeft() -> Element {
    let state = use_context::<AppState>();
    let mut group_open = use_signal(|| false);
    let mut group_name = use_signal(String::new);
    let any_open = {
        let open_map = state.task_open.read();
        state
            .groups_list()
            .iter()
            .any(|g| open_map.get(&g.id).copied().unwrap_or(true))
    };
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
        div { class: "ds-menuwrap",
            button {
                class: "bg-btn ghost",
                title: "Új csoport",
                style: "width: 44px; padding: 0; justify-content: center;",
                onclick: move |_| {
                    let v = group_open();
                    group_open.set(!v);
                },
                Icon { name: "folder", size: 17.0 }
            }
            if group_open() {
                div { class: "ds-menu-scrim", onclick: move |_| group_open.set(false) }
                div { class: "ds-menu", style: "min-width: 250px; left: 0; right: auto;",
                    div { style: "display: flex; gap: 8px; padding: 4px;",
                        input {
                            class: "bg-input",
                            placeholder: "Csoport neve…",
                            value: "{group_name}",
                            oninput: move |e| group_name.set(e.value()),
                        }
                        button {
                            class: "bg-btn sm",
                            onclick: move |_| {
                                let name = group_name();
                                if name.trim().is_empty() {
                                    return;
                                }
                                group_open.set(false);
                                group_name.set(String::new());
                                let mut state = state;
                                spawn(async move {
                                    if api::tasks::create_group(name).await.is_ok() {
                                        state.groups.restart();
                                    }
                                });
                            },
                            Icon { name: "check", size: 14.0 }
                        }
                    }
                }
            }
        }
        button {
            class: "bg-iconbtn",
            title: if any_open { "Összes csoport összecsukása" } else { "Összes csoport kinyitása" },
            onclick: move |_| {
                let groups = state.groups_list();
                let mut open = state.task_open;
                let target = {
                    let map = open.read();
                    !groups.iter().any(|g| map.get(&g.id).copied().unwrap_or(true))
                };
                open.set(groups.iter().map(|g| (g.id, target)).collect());
            },
            Icon { name: if any_open { "collapseall" } else { "expandall" }, size: 18.0, stroke: 2.0 }
        }
    }
}

#[component]
pub fn TasksHeaderRight() -> Element {
    let state = use_context::<AppState>();
    let filter = (state.task_filter)();
    let groups = state.groups_list();
    let total: usize = groups.iter().map(|g| g.tasks.len()).sum();
    let open: usize = groups
        .iter()
        .map(|g| g.tasks.iter().filter(|t| !t.done).count())
        .sum();
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
fn SubRow(sub: SubTaskDto) -> Element {
    let state = use_context::<AppState>();
    let sub_id = sub.id;
    let done = sub.done;
    rsx! {
        div { class: if done { "tk-subrow done" } else { "tk-subrow" },
            button {
                class: if done { "tk-check done" } else { "tk-check" },
                onclick: move |e| {
                    e.stop_propagation();
                    let mut state = state;
                    spawn(async move {
                        if api::tasks::set_subtask_done(sub_id, !done).await.is_ok() {
                            state.groups.restart();
                        }
                    });
                },
                if done {
                    Icon { name: "checkmini", size: 11.0, stroke: 2.6 }
                }
            }
            span { class: "tx", style: "flex: 1;", "{sub.title}" }
        }
    }
}

#[component]
fn TaskMenu(task: TaskDto, onclose: EventHandler<()>) -> Element {
    let state = use_context::<AppState>();
    let task_id = task.id;
    let done = task.done;
    rsx! {
        div { class: "ds-menu-scrim", onclick: move |_| onclose.call(()) }
        div { class: "ds-menu", onclick: move |e| e.stop_propagation(),
            button {
                class: "ds-menu-item",
                onclick: move |_| {
                    onclose.call(());
                    let mut state = state;
                    spawn(async move {
                        if api::tasks::set_task_done(task_id, !done).await.is_ok() {
                            state.groups.restart();
                        }
                    });
                },
                Icon { name: "check", size: 16.0 }
                if done { " Újranyitás" } else { " Késznek jelöl" }
            }
            button {
                class: "ds-menu-item",
                onclick: move |_| {
                    onclose.call(());
                    let mut p = state.task_panel;
                    p.set(Some(TaskPanel::Edit(task_id)));
                },
                Icon { name: "settings", size: 16.0 }
                " Szerkesztés"
            }
            div { class: "ds-menu-sep" }
            button {
                class: "ds-menu-item danger",
                onclick: move |_| {
                    onclose.call(());
                    let mut state = state;
                    spawn(async move {
                        if api::tasks::delete_task(task_id).await.is_ok() {
                            state.groups.restart();
                        }
                    });
                },
                Icon { name: "x", size: 16.0 }
                " Törlés"
            }
        }
    }
}

#[component]
fn TaskItem(task: TaskDto) -> Element {
    let state = use_context::<AppState>();
    let mut open = use_signal(|| false);
    let mut menu = use_signal(|| false);
    let task_id = task.id;
    let is_done = task.done;
    let is_open = open();
    let sub_done = task.subs.iter().filter(|s| s.done).count();
    let has_subs = !task.subs.is_empty();
    let assignee = task.assignee.map(|id| state.user(id));
    let menu_task = task.clone();

    rsx! {
        div { class: if is_done { "tk-item done" } else { "tk-item" },
            div {
                class: "tk-row tk-clickable",
                onclick: move |_| {
                    let mut p = state.task_panel;
                    p.set(Some(TaskPanel::Edit(task_id)));
                },
                button {
                    class: if is_done { "tk-check done" } else { "tk-check" },
                    onclick: move |e| {
                        e.stop_propagation();
                        let mut state = state;
                        spawn(async move {
                            if api::tasks::set_task_done(task_id, !is_done).await.is_ok() {
                                state.groups.restart();
                                state.threads.restart();
                            }
                        });
                    },
                    if is_done {
                        Icon { name: "check", size: 14.0, stroke: 2.6 }
                    }
                }
                div { class: "tk-main",
                    div { class: "tk-title",
                        span { class: "txt", "{task.title}" }
                        if let Some(rec) = &task.recurring {
                            span { class: "bg-chip reed", style: "height: 22px;",
                                Icon { name: "repeat", size: 13.0 }
                                " {recurring_label(rec)}"
                            }
                        }
                    }
                    div { class: "tk-meta",
                        if let Some(u) = assignee {
                            span { class: "m",
                                Avatar { user: u.clone(), size: "sm" }
                                " {u.name}"
                            }
                        }
                        if let Some(due) = &task.due {
                            span { class: "m",
                                Icon { name: "clock" }
                                " {hu_date(due)}"
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
                                state.go_discuss("task", task_id);
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
                        TaskMenu { task: menu_task.clone(), onclose: move |_| menu.set(false) }
                    }
                }
            }
            if has_subs && is_open {
                div { class: "tk-sub",
                    for s in task.subs.clone() {
                        SubRow { key: "{s.id}", sub: s }
                    }
                }
            }
        }
    }
}

#[component]
fn TaskGroupView(group_id: i64, name: String, tasks: Vec<TaskDto>, all_count: usize, open_count: usize, open: bool) -> Element {
    let state = use_context::<AppState>();
    rsx! {
        div { class: if open { "tk-group" } else { "tk-group closed" },
            button {
                class: "tk-ghead",
                onclick: move |_| {
                    let mut m = state.task_open;
                    let v = m.read().get(&group_id).copied().unwrap_or(true);
                    m.write().insert(group_id, !v);
                },
                Icon { name: "chevdown", size: 16.0, stroke: 2.2, class: "gchev" }
                Icon { name: "folder", size: 17.0, stroke: 2.0, style: "color: var(--water);" }
                span { class: "gname", "{name}" }
                span { class: "gcount", "{open_count}/{all_count}" }
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
    let groups = state.groups_list();
    let task: Option<TaskDto> = match &panel {
        TaskPanel::New => None,
        TaskPanel::Edit(id) => groups
            .iter()
            .flat_map(|g| g.tasks.iter())
            .find(|t| t.id == *id)
            .cloned(),
    };
    let is_new = task.is_none();
    let task_id = task.as_ref().map(|t| t.id);
    let me_id = state.me().id;

    let mut title = use_signal(|| task.as_ref().map(|t| t.title.clone()).unwrap_or_default());
    let mut group_id = use_signal(|| {
        task.as_ref()
            .map(|t| t.group_id)
            .or_else(|| groups.first().map(|g| g.id))
            .unwrap_or(0)
    });
    let mut assignee = use_signal(|| task.as_ref().and_then(|t| t.assignee).or(Some(me_id)));
    let mut due = use_signal(|| task.as_ref().and_then(|t| t.due.clone()).unwrap_or_default());
    let mut recurring = use_signal(|| {
        task.as_ref()
            .and_then(|t| t.recurring.clone())
            .unwrap_or_default()
    });
    let mut new_sub = use_signal(String::new);
    let mut event_from = use_signal(String::new);
    let mut event_to = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut busy = use_signal(|| false);

    let close = move |_: MouseEvent| {
        let mut p = state.task_panel;
        p.set(None);
    };

    let save = move |_: MouseEvent| {
        if busy() {
            return;
        }
        busy.set(true);
        error.set(String::new());
        let input = TaskInput {
            group_id: group_id(),
            title: title(),
            due: Some(due()).filter(|d| !d.is_empty()),
            assignee: assignee(),
            recurring: Some(recurring()).filter(|r| !r.is_empty()),
        };
        let mut state = state;
        spawn(async move {
            let result = match task_id {
                Some(id) => api::tasks::update_task(id, input).await,
                None => api::tasks::create_task(input).await.map(|_| ()),
            };
            match result {
                Ok(()) => {
                    state.groups.restart();
                    let mut p = state.task_panel;
                    p.set(None);
                }
                Err(e) => error.set(clean_err(&e.to_string())),
            }
            busy.set(false);
        });
    };

    let done = task.as_ref().map(|t| t.done).unwrap_or(false);
    let group_name = groups
        .iter()
        .find(|g| g.id == group_id())
        .map(|g| g.name.clone())
        .unwrap_or_else(|| "Új feladat".into());
    let users = state.users_list();
    let rec = recurring();

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
                ErrorNote { message: error() }
                div { class: "bg-field",
                    label { "Megnevezés" }
                    input {
                        class: "bg-input",
                        placeholder: "Mi a teendő?",
                        value: "{title}",
                        oninput: move |e| title.set(e.value()),
                    }
                }
                div { class: "bg-field",
                    label { "Csoport" }
                    select {
                        class: "bg-input",
                        value: "{group_id}",
                        onchange: move |e| {
                            if let Ok(id) = e.value().parse() {
                                group_id.set(id);
                            }
                        },
                        for g in groups.iter() {
                            option { value: "{g.id}", selected: g.id == group_id(), "{g.name}" }
                        }
                    }
                }
                div { class: "set-pwgrid",
                    div { class: "bg-field",
                        label { "Felelős" }
                        select {
                            class: "bg-input",
                            onchange: move |e| {
                                assignee.set(e.value().parse::<i64>().ok().filter(|id| *id > 0));
                            },
                            option { value: "0", selected: assignee().is_none(), "Nincs" }
                            for u in users.iter() {
                                option { value: "{u.id}", selected: assignee() == Some(u.id), "{u.name}" }
                            }
                        }
                    }
                    div { class: "bg-field",
                        label { "Határidő" }
                        input {
                            class: "bg-input",
                            r#type: "date",
                            value: "{due}",
                            oninput: move |e| due.set(e.value()),
                        }
                    }
                }
                div { class: "bg-field",
                    label { "Ismétlődés" }
                    div { class: "bg-seg",
                        button {
                            class: if rec.is_empty() { "on" } else { "" },
                            onclick: move |_| recurring.set(String::new()),
                            "Egyszeri"
                        }
                        for (key, label) in RECURRING_OPTIONS.iter().take(3) {
                            button {
                                key: "{key}",
                                class: if rec == *key { "on" } else { "" },
                                onclick: move |_| recurring.set(key.to_string()),
                                "{label}"
                            }
                        }
                    }
                }
                if let Some(t) = task.clone() {
                    div { class: "bg-field",
                        label { "Alfeladatok · {t.subs.iter().filter(|s| s.done).count()}/{t.subs.len()}" }
                        div {
                            for s in t.subs.clone() {
                                SubRow { key: "{s.id}", sub: s }
                            }
                        }
                        div { style: "display: flex; gap: 8px; margin-top: 9px;",
                            input {
                                class: "bg-input",
                                placeholder: "Új alfeladat…",
                                value: "{new_sub}",
                                oninput: move |e| new_sub.set(e.value()),
                            }
                            button {
                                class: "bg-btn ghost sm",
                                style: "height: 40px;",
                                onclick: move |_| {
                                    let sub_title = new_sub();
                                    if sub_title.trim().is_empty() {
                                        return;
                                    }
                                    new_sub.set(String::new());
                                    let mut state = state;
                                    let tid = t.id;
                                    spawn(async move {
                                        if api::tasks::add_subtask(tid, sub_title).await.is_ok() {
                                            state.groups.restart();
                                        }
                                    });
                                },
                                Icon { name: "plus", size: 14.0 }
                            }
                        }
                    }
                    if let Some(ev) = t.event.clone() {
                        div { class: "bg-field",
                            label { "Kapcsolt esemény" }
                            {
                                let res_id = ev.res_id;
                                rsx! {
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
                    } else {
                        div { class: "bg-field",
                            label { "Esemény kapcsolása" }
                            p { style: "font-size: 12.5px; color: var(--ink-3); margin-bottom: 8px;",
                                "Nyitott foglalást hoz létre a feladathoz a megadott napokra."
                            }
                            div { class: "set-pwgrid",
                                input {
                                    class: "bg-input",
                                    r#type: "date",
                                    value: "{event_from}",
                                    oninput: move |e| event_from.set(e.value()),
                                }
                                input {
                                    class: "bg-input",
                                    r#type: "date",
                                    value: "{event_to}",
                                    oninput: move |e| event_to.set(e.value()),
                                }
                            }
                            button {
                                class: "bg-btn ghost sm",
                                style: "margin-top: 9px;",
                                onclick: move |_| {
                                    let from = event_from();
                                    let to = event_to();
                                    if from.is_empty() || to.is_empty() {
                                        error.set("Add meg az esemény napjait.".into());
                                        return;
                                    }
                                    let mut state = state;
                                    let tid = t.id;
                                    spawn(async move {
                                        match api::tasks::attach_event(tid, from, to).await {
                                            Ok(_) => {
                                                state.groups.restart();
                                                state.reservations.restart();
                                            }
                                            Err(e) => error.set(clean_err(&e.to_string())),
                                        }
                                    });
                                },
                                Icon { name: "calendar", size: 14.0 }
                                " Esemény létrehozása"
                            }
                        }
                    }
                }
            }
            div { class: "pfoot",
                if let Some(id) = task_id {
                    button {
                        class: "bg-btn ghost",
                        style: "flex: 1; justify-content: center;",
                        onclick: move |_| state.go_discuss("task", id),
                        Icon { name: "chat", size: 16.0 }
                        " Beszélgetés"
                    }
                }
                button {
                    class: "bg-btn",
                    style: "flex: 1; justify-content: center;",
                    disabled: busy(),
                    onclick: save,
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
    let groups = state.groups_list();
    let open_map = state.task_open.read().clone();

    let match_filter = |t: &TaskDto| match filter {
        "active" => !t.done,
        "done" => t.done,
        _ => true,
    };
    let shown: Vec<(i64, String, Vec<TaskDto>, usize, usize)> = groups
        .iter()
        .map(|g| {
            (
                g.id,
                g.name.clone(),
                g.tasks.iter().filter(|t| match_filter(t)).cloned().collect::<Vec<_>>(),
                g.tasks.len(),
                g.tasks.iter().filter(|t| !t.done).count(),
            )
        })
        .filter(|(_, _, tasks, _, _)| !tasks.is_empty())
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
                    if shown.is_empty() {
                        div { class: "tk-empty",
                            Icon { name: "check", size: 26.0, stroke: 2.0 }
                            if groups.is_empty() {
                                " Még nincs feladat — hozz létre egy csoportot és egy feladatot."
                            } else {
                                " Nincs ilyen feladat."
                            }
                        }
                    } else {
                        for (gid, name, tasks, all_count, open_count) in shown {
                            TaskGroupView {
                                key: "{gid}",
                                group_id: gid,
                                name,
                                tasks,
                                all_count,
                                open_count,
                                open: open_map.get(&gid).copied().unwrap_or(true),
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

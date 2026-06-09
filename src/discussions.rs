//! Discussions: threads, sub-threads, votes, pins, polls, closed threads.

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::common::{Avatar, AvatarStack};
use crate::data::{user, Msg, Poll, Thread, ME, THREADS, THREAD_FILTERS};
use crate::icons::Icon;
use crate::state::AppState;

fn kind_icon(kind: &str) -> &'static str {
    match kind {
        "reservation" => "calendar",
        "task" => "tasks",
        _ => "chat",
    }
}

fn filtered(filter: &str) -> Vec<&'static Thread> {
    THREADS
        .iter()
        .filter(|t| filter == "all" || t.kind == filter)
        .collect()
}

/// The thread shown for the current state: an explicit pick, or the first
/// thread on desktop (mobile starts on the list).
fn effective_thread(state: &AppState, mobile: bool) -> Option<&'static Thread> {
    let picked = (state.active_thread)();
    match picked {
        Some(id) => THREADS.iter().find(|t| t.id == id),
        None if !mobile => Some(&THREADS[0]),
        None => None,
    }
}

#[component]
pub fn DiscHeaderLeft() -> Element {
    let state = use_context::<AppState>();
    let filter = (state.disc_filter)();
    rsx! {
        div { class: "ds-filter-tabs",
            for (id, label) in THREAD_FILTERS {
                button {
                    key: "{id}",
                    class: if filter == *id { "on" } else { "" },
                    onclick: move |_| {
                        let mut f = state.disc_filter;
                        f.set(id);
                    },
                    "{label}"
                }
            }
        }
        button { class: "ds-newthread", title: "Új beszélgetés",
            Icon { name: "plus", size: 17.0 }
        }
    }
}

#[component]
pub fn DiscHeaderRight() -> Element {
    let state = use_context::<AppState>();
    let count = filtered((state.disc_filter)()).len();
    rsx! {
        span { class: "bg-chip",
            Icon { name: "chat", size: 14.0 }
            " {count} téma"
        }
    }
}

#[component]
fn VoteBox(up: u32, down: u32) -> Element {
    let mut v = use_signal(|| 0i8);
    let cur = v();
    let up_n = up + if cur == 1 { 1 } else { 0 };
    let down_n = down + if cur == -1 { 1 } else { 0 };
    rsx! {
        div { class: "ds-vote",
            button {
                class: if cur == 1 { "vb up on" } else { "vb up" },
                title: "Egyetértek",
                onclick: move |_| {
                    let x = v();
                    v.set(if x == 1 { 0 } else { 1 });
                },
                Icon { name: "arrowup", size: 15.0, stroke: 2.4 }
                span { class: "vc", "{up_n}" }
            }
            button {
                class: if cur == -1 { "vb down on" } else { "vb down" },
                title: "Nem értek egyet",
                onclick: move |_| {
                    let x = v();
                    v.set(if x == -1 { 0 } else { -1 });
                },
                Icon { name: "arrowdown", size: 15.0, stroke: 2.4 }
                span { class: "vc", "{down_n}" }
            }
        }
    }
}

#[component]
fn PollCard(poll: &'static Poll) -> Element {
    let mut picks = use_signal(|| {
        poll.options
            .iter()
            .map(|o| (o.id, o.votes.contains(&ME)))
            .collect::<HashMap<&'static str, bool>>()
    });

    let picks_now = picks.read().clone();
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut max = 1usize;
    for o in poll.options {
        let base = o.votes.iter().filter(|u| **u != ME).count();
        let c = base + if picks_now.get(o.id).copied().unwrap_or(false) { 1 } else { 0 };
        if c > max {
            max = c;
        }
        counts.insert(o.id, c);
    }
    let total: usize = counts.values().sum();

    rsx! {
        div { class: "ds-poll",
            div { class: "ds-poll-head",
                Icon { name: if poll.ptype == "date" { "calendar" } else { "list" }, size: 15.0, stroke: 2.2 }
                span { class: "q", "{poll.question}" }
                span { class: "ds-poll-tag",
                    if poll.mode == "single" { "Egy választás" } else { "Több is választható" }
                }
            }
            div { class: "ds-poll-opts",
                for o in poll.options {
                    {
                        let on = picks_now.get(o.id).copied().unwrap_or(false);
                        let c = counts.get(o.id).copied().unwrap_or(0);
                        let voters: Vec<String> = o
                            .votes
                            .iter()
                            .filter(|u| **u != ME)
                            .map(|u| u.to_string())
                            .chain(if on { vec![ME.to_string()] } else { vec![] })
                            .collect();
                        let oid = o.id;
                        let width = c as f64 / max as f64 * 100.0;
                        rsx! {
                            button {
                                key: "{o.id}",
                                class: if on { "ds-poll-opt on" } else { "ds-poll-opt" },
                                onclick: move |_| {
                                    let mut p = picks.write();
                                    if poll.mode == "single" {
                                        let was = p.get(oid).copied().unwrap_or(false);
                                        for opt in poll.options {
                                            p.insert(opt.id, false);
                                        }
                                        p.insert(oid, !was);
                                    } else {
                                        let was = p.get(oid).copied().unwrap_or(false);
                                        p.insert(oid, !was);
                                    }
                                },
                                span {
                                    class: {
                                        let mut c = String::from("ds-poll-ctrl");
                                        if poll.mode == "single" { c.push_str(" radio"); }
                                        if on { c.push_str(" on"); }
                                        c
                                    },
                                    if on {
                                        Icon { name: "checkmini", size: 11.0, stroke: 3.0 }
                                    }
                                }
                                span { class: "ds-poll-bar", style: "width: {width}%;" }
                                span { class: "ds-poll-label",
                                    "{o.label}"
                                    if !o.sub.is_empty() {
                                        span { class: "sub", " · {o.sub}" }
                                    }
                                }
                                span { class: "ds-poll-voters",
                                    if !voters.is_empty() {
                                        AvatarStack { ids: voters, size: "sm", max: 3 }
                                    }
                                    span { class: "n", "{c}" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "ds-poll-foot", "{total} szavazat · a részvétel látható mindenkinek" }
        }
    }
}

#[component]
fn Message(msg: &'static Msg) -> Element {
    if msg.system {
        return rsx! {
            div { class: "ds-sys",
                Icon { name: "info", size: 14.0 }
                " {msg.text} · {msg.time}"
            }
        };
    }
    let author = user(msg.author);
    rsx! {
        div { class: if msg.pinned { "ds-msg pinned" } else { "ds-msg" },
            Avatar { id: "{msg.author}" }
            div { class: "body",
                div { class: "ds-bubble",
                    div { class: "ds-mhead",
                        span { class: "au", "{author.name}" }
                        if author.approver {
                            span { class: "bg-chip reed", style: "height: 19px; font-size: 11px; padding: 0 7px;", "Engedélyező" }
                        }
                        span { class: "tm", "{msg.time}" }
                        if msg.pinned {
                            span { class: "pinflag",
                                Icon { name: "pin", size: 13.0, fill: true }
                                " Kitűzve"
                            }
                        }
                    }
                    if let Some(poll) = &msg.poll {
                        PollCard { poll }
                    } else {
                        div { class: "ds-mtext", "{msg.text}" }
                    }
                    if !msg.image.is_empty() {
                        div { class: "ph-img", style: "height: 130px; margin-top: 10px;", "{msg.image}" }
                    }
                }
                div { class: "ds-mactions",
                    if msg.poll.is_none() {
                        VoteBox { up: msg.votes, down: msg.down }
                    }
                    button { class: "ds-tinybtn",
                        Icon { name: "reply", size: 15.0 }
                        " Válasz"
                    }
                    button { class: "ds-tinybtn",
                        Icon { name: "pin", size: 15.0 }
                        if msg.pinned { " Levesz" } else { " Kitűz" }
                    }
                }
                if !msg.replies.is_empty() {
                    div { class: "ds-replies",
                        for r in msg.replies {
                            Message { key: "{r.id}", msg: r }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PollBuilder(oncancel: EventHandler<()>) -> Element {
    let mut ptype = use_signal(|| "date");
    let mut mode = use_signal(|| "single");
    let mut opts = use_signal(|| vec![String::new(), String::new()]);
    let n_opts = opts.read().len();
    rsx! {
        div { class: "ds-pollbuild",
            div { class: "pb-head",
                Icon { name: "list", size: 16.0, stroke: 2.2 }
                " "
                b { "Új szavazás" }
                button {
                    class: "bg-iconbtn",
                    style: "margin-left: auto; width: 32px; height: 32px;",
                    onclick: move |_| oncancel.call(()),
                    Icon { name: "x", size: 16.0 }
                }
            }
            input { class: "bg-input", placeholder: "Kérdés — pl. Melyik hétvégén?" }
            div { class: "set-pwgrid",
                div { class: "bg-field",
                    label { "Típus" }
                    div { class: "bg-seg",
                        button {
                            class: if ptype() == "date" { "on" } else { "" },
                            onclick: move |_| ptype.set("date"),
                            Icon { name: "calendar", size: 14.0 }
                            " Dátum"
                        }
                        button {
                            class: if ptype() == "list" { "on" } else { "" },
                            onclick: move |_| ptype.set("list"),
                            Icon { name: "list", size: 14.0 }
                            " Lista"
                        }
                    }
                }
                div { class: "bg-field",
                    label { "Választás" }
                    div { class: "bg-seg",
                        button {
                            class: if mode() == "single" { "on" } else { "" },
                            onclick: move |_| mode.set("single"),
                            "Egy"
                        }
                        button {
                            class: if mode() == "multi" { "on" } else { "" },
                            onclick: move |_| mode.set("multi"),
                            "Több"
                        }
                    }
                }
            }
            div { class: "bg-field",
                label { "Opciók" }
                div { class: "pb-opts",
                    for i in 0..n_opts {
                        div { class: "pb-optrow", key: "{i}",
                            span { class: if mode() == "single" { "ds-poll-ctrl radio" } else { "ds-poll-ctrl" } }
                            input {
                                class: "bg-input",
                                placeholder: if ptype() == "date" { "pl. Szombat, máj. 30." } else { "Opció szövege…" },
                            }
                            if n_opts > 2 {
                                button {
                                    class: "bg-iconbtn",
                                    style: "width: 34px; height: 34px;",
                                    onclick: move |_| {
                                        opts.write().remove(i);
                                    },
                                    Icon { name: "x", size: 15.0 }
                                }
                            }
                        }
                    }
                }
                button {
                    class: "bg-btn ghost sm",
                    style: "margin-top: 9px;",
                    onclick: move |_| opts.write().push(String::new()),
                    Icon { name: "plus", size: 14.0 }
                    " Opció hozzáadása"
                }
            }
            div { style: "display: flex; gap: 10px;",
                button {
                    class: "bg-btn",
                    style: "flex: 1; justify-content: center;",
                    onclick: move |_| oncancel.call(()),
                    Icon { name: "check", size: 16.0 }
                    " Szavazás indítása"
                }
                button { class: "bg-btn ghost", onclick: move |_| oncancel.call(()), "Mégse" }
            }
        }
    }
}

#[component]
fn ThreadView(thread: &'static Thread) -> Element {
    let state = use_context::<AppState>();
    let device_mobile = (state.is_mobile)();
    let mut menu = use_signal(|| false);
    let mut compose_poll = use_signal(|| false);
    let author = user(thread.author);
    let (kind_ic, kind_label, kind_cls) = match thread.kind {
        "reservation" => ("calendar", "Foglalás", "closed"),
        "task" => ("tasks", "Feladat", "reed"),
        _ => ("chat", "Általános", ""),
    };
    let _ = kind_ic;

    rsx! {
        div { class: "ds-thread-view bg-fade",
            div { class: "ds-thead",
                if device_mobile {
                    button {
                        class: "bg-iconbtn",
                        style: "flex-shrink: 0;",
                        onclick: move |_| {
                            let mut a = state.active_thread;
                            a.set(None);
                        },
                        Icon { name: "chevleft", size: 18.0 }
                    }
                }
                div { style: "flex: 1; min-width: 0;",
                    h2 { "{thread.title}" }
                    div { class: "ds-tmeta",
                        span { class: "bg-chip",
                            Avatar { id: "{thread.author}", size: "sm" }
                            " {author.name}"
                        }
                        if !thread.link_id.is_empty() {
                            button {
                                class: "bg-chip linkchip {kind_cls}",
                                title: if thread.kind == "task" { "Ugrás a feladatra" } else { "Ugrás a foglalásra" },
                                onclick: move |_| state.open_link(thread.kind, thread.link_id),
                                Icon { name: "link", size: 13.0 }
                                span { class: "lc-txt", " {kind_label}: {thread.link_label} " }
                                Icon { name: "chevright", size: 13.0 }
                            }
                        }
                        if thread.closed {
                            span { class: "bg-chip reject",
                                Icon { name: "lock", size: 13.0 }
                                " Lezárva"
                            }
                        }
                    }
                }
                div { class: "ds-menuwrap",
                    button {
                        class: if menu() { "bg-iconbtn on" } else { "bg-iconbtn" },
                        style: "flex-shrink: 0;",
                        onclick: move |_| {
                            let v = menu();
                            menu.set(!v);
                        },
                        Icon { name: "dots", size: 18.0 }
                    }
                    if menu() {
                        div { class: "ds-menu-scrim", onclick: move |_| menu.set(false) }
                        div { class: "ds-menu",
                            button { class: "ds-menu-item",
                                Icon { name: "pin", size: 16.0 }
                                " Téma kitűzése"
                            }
                            button { class: "ds-menu-item",
                                Icon { name: "bell", size: 16.0 }
                                " Némítás"
                            }
                            button { class: "ds-menu-item",
                                Icon { name: "link", size: 16.0 }
                                " Megosztás…"
                            }
                            button { class: "ds-menu-item",
                                Icon { name: "lock", size: 16.0 }
                                " Beszélgetés lezárása"
                            }
                            div { class: "ds-menu-sep" }
                            button { class: "ds-menu-item danger",
                                Icon { name: "x", size: 16.0 }
                                " Törlés"
                            }
                        }
                    }
                }
            }
            div { class: "ds-scroll",
                for m in thread.messages {
                    Message { key: "{m.id}", msg: m }
                }
            }
            if thread.closed {
                div {
                    class: "ds-composer",
                    style: "justify-content: center; color: var(--ink-3); font-size: 13.5px; font-weight: 600; gap: 8px;",
                    Icon { name: "lock", size: 15.0 }
                    " Ezt a beszélgetést lezárták — csak engedélyező nyithatja meg újra."
                }
            } else if compose_poll() {
                PollBuilder { oncancel: move |_| compose_poll.set(false) }
            } else {
                div { class: "ds-composer",
                    Avatar { id: "{ME}" }
                    textarea { class: "bg-input ds-input", placeholder: "Írj egy üzenetet…" }
                    button {
                        class: "bg-iconbtn ds-pollbtn",
                        title: "Szavazás létrehozása",
                        onclick: move |_| compose_poll.set(true),
                        Icon { name: "list", size: 18.0 }
                    }
                    button { class: "bg-btn", style: "flex-shrink: 0; width: 44px; padding: 0; justify-content: center;",
                        Icon { name: "send", size: 18.0, fill: true }
                    }
                }
            }
        }
    }
}

#[component]
pub fn DiscussionsTool() -> Element {
    let state = use_context::<AppState>();
    let device_mobile = (state.is_mobile)();
    let filter = (state.disc_filter)();
    let list = filtered(filter);
    let active = effective_thread(&state, device_mobile);
    let show_thread = device_mobile && active.is_some();

    rsx! {
        div { class: "bg-content", style: "overflow: hidden; padding: 0;",
            div { class: if show_thread { "ds-cols show-thread" } else { "ds-cols" },
                div { class: "ds-list",
                    if device_mobile {
                        div { class: "tool-toolbar",
                            div { class: "tt-left", DiscHeaderLeft {} }
                            div { class: "tt-right", DiscHeaderRight {} }
                        }
                    }
                    for t in list {
                        div {
                            key: "{t.id}",
                            class: if active.map(|a| a.id == t.id).unwrap_or(false) && !device_mobile { "ds-thread active" } else { "ds-thread" },
                            onclick: move |_| {
                                let mut a = state.active_thread;
                                a.set(Some(t.id));
                            },
                            div { style: "display: flex; align-items: center; gap: 8px;",
                                Icon { name: "{kind_icon(t.kind)}", size: 15.0, stroke: 2.0, style: "color: var(--water); flex-shrink: 0;" }
                                span { class: "tt", style: "flex: 1;", "{t.title}" }
                                if t.closed {
                                    Icon { name: "lock", size: 14.0, style: "color: var(--ink-3);" }
                                }
                            }
                            div { class: "ex", "{t.excerpt}" }
                            div { class: "mt",
                                Avatar { id: "{t.author}", size: "sm" }
                                span { "{user(t.author).name}" }
                                span { "· {t.time}" }
                                span { style: "margin-left: auto; display: flex; gap: 10px;",
                                    span { style: "display: flex; align-items: center; gap: 4px;",
                                        Icon { name: "chat", size: 13.0 }
                                        " {t.replies}"
                                    }
                                    span { style: "display: flex; align-items: center; gap: 4px;",
                                        Icon { name: "arrowup", size: 13.0 }
                                        " {t.votes}"
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(t) = active {
                    ThreadView { thread: t }
                }
            }
        }
    }
}

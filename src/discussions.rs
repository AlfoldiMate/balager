//! Discussions: threads, sub-threads, votes, pins, polls — live data.

use dioxus::prelude::*;

use crate::api;
use crate::common::{Avatar, AvatarStack, ErrorNote};
use crate::icons::Icon;
use crate::login::clean_err;
use crate::models::{time_label, MessageDto, PollDto, ThreadDetailDto, ThreadDto};
use crate::state::AppState;

pub static THREAD_FILTERS: &[(&str, &str)] = &[
    ("all", "Mind"),
    ("general", "Általános"),
    ("reservation", "Foglalás"),
    ("task", "Feladat"),
];

fn kind_icon(kind: &str) -> &'static str {
    match kind {
        "reservation" => "calendar",
        "task" => "tasks",
        _ => "chat",
    }
}

fn filtered(state: &AppState) -> Vec<ThreadDto> {
    let filter = (state.disc_filter)();
    state
        .threads_list()
        .into_iter()
        .filter(|t| filter == "all" || t.kind == filter)
        .collect()
}

/// The thread shown: explicit pick, or the first thread on desktop.
fn effective_thread(state: &AppState, mobile: bool) -> Option<i64> {
    if (state.disc_creating)() {
        return None;
    }
    match (state.active_thread)() {
        Some(id) => Some(id),
        None if !mobile => filtered(state).first().map(|t| t.id),
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
        button {
            class: "ds-newthread",
            title: "Új beszélgetés",
            onclick: move |_| {
                let mut c = state.disc_creating;
                c.set(true);
                let mut a = state.active_thread;
                a.set(None);
            },
            Icon { name: "plus", size: 17.0 }
        }
    }
}

#[component]
pub fn DiscHeaderRight() -> Element {
    let state = use_context::<AppState>();
    let count = filtered(&state).len();
    rsx! {
        span { class: "bg-chip",
            Icon { name: "chat", size: 14.0 }
            " {count} téma"
        }
    }
}

#[component]
fn VoteBox(message_id: i64, up: i64, down: i64, my_vote: i64, onvoted: EventHandler<()>) -> Element {
    let vote = move |value: i64| {
        let target = if my_vote == value { 0 } else { value };
        spawn(async move {
            if api::discussions::vote_message(message_id, target).await.is_ok() {
                onvoted.call(());
            }
        });
    };
    rsx! {
        div { class: "ds-vote",
            button {
                class: if my_vote == 1 { "vb up on" } else { "vb up" },
                title: "Egyetértek",
                onclick: move |_| vote(1),
                Icon { name: "arrowup", size: 15.0, stroke: 2.4 }
                span { class: "vc", "{up}" }
            }
            button {
                class: if my_vote == -1 { "vb down on" } else { "vb down" },
                title: "Nem értek egyet",
                onclick: move |_| vote(-1),
                Icon { name: "arrowdown", size: 15.0, stroke: 2.4 }
                span { class: "vc", "{down}" }
            }
        }
    }
}

#[component]
fn PollCard(poll: PollDto, onvoted: EventHandler<()>) -> Element {
    let state = use_context::<AppState>();
    let me = state.me().id;
    let max = poll.options.iter().map(|o| o.votes.len()).max().unwrap_or(0).max(1);
    let total: usize = poll.options.iter().map(|o| o.votes.len()).sum();
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
                for o in poll.options.clone() {
                    {
                        let on = o.votes.contains(&me);
                        let count = o.votes.len();
                        let width = count as f64 / max as f64 * 100.0;
                        let voters: Vec<_> = o.votes.iter().map(|id| state.user(*id)).collect();
                        let oid = o.id;
                        let single = poll.mode == "single";
                        rsx! {
                            button {
                                key: "{o.id}",
                                class: if on { "ds-poll-opt on" } else { "ds-poll-opt" },
                                onclick: move |_| {
                                    spawn(async move {
                                        if api::discussions::poll_vote(oid, !on).await.is_ok() {
                                            onvoted.call(());
                                        }
                                    });
                                },
                                span {
                                    class: {
                                        let mut c = String::from("ds-poll-ctrl");
                                        if single { c.push_str(" radio"); }
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
                                        AvatarStack { users: voters, size: "sm", max: 3 }
                                    }
                                    span { class: "n", "{count}" }
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
fn Message(
    msg: MessageDto,
    closed: bool,
    onreply: EventHandler<i64>,
    onchanged: EventHandler<()>,
) -> Element {
    let state = use_context::<AppState>();
    if msg.system {
        return rsx! {
            div { class: "ds-sys",
                Icon { name: "info", size: 14.0 }
                " {msg.body} · {time_label(msg.created_at)}"
            }
        };
    }
    let author = state.user(msg.author.unwrap_or(0));
    let msg_id = msg.id;
    let pinned = msg.pinned;
    rsx! {
        div { class: if pinned { "ds-msg pinned" } else { "ds-msg" },
            Avatar { user: author.clone() }
            div { class: "body",
                div { class: "ds-bubble",
                    div { class: "ds-mhead",
                        span { class: "au", "{author.name}" }
                        if author.approver {
                            span { class: "bg-chip reed", style: "height: 19px; font-size: 11px; padding: 0 7px;", "Engedélyező" }
                        }
                        span { class: "tm", "{time_label(msg.created_at)}" }
                        if pinned {
                            span { class: "pinflag",
                                Icon { name: "pin", size: 13.0, fill: true }
                                " Kitűzve"
                            }
                        }
                    }
                    if let Some(poll) = msg.poll.clone() {
                        PollCard { poll, onvoted: move |_| onchanged.call(()) }
                    } else {
                        div { class: "ds-mtext", "{msg.body}" }
                    }
                }
                div { class: "ds-mactions",
                    if msg.poll.is_none() {
                        VoteBox {
                            message_id: msg_id,
                            up: msg.up,
                            down: msg.down,
                            my_vote: msg.my_vote,
                            onvoted: move |_| onchanged.call(()),
                        }
                    }
                    if !closed {
                        button { class: "ds-tinybtn", onclick: move |_| onreply.call(msg_id),
                            Icon { name: "reply", size: 15.0 }
                            " Válasz"
                        }
                    }
                    button {
                        class: "ds-tinybtn",
                        onclick: move |_| {
                            spawn(async move {
                                if api::discussions::set_pinned(msg_id, !pinned).await.is_ok() {
                                    onchanged.call(());
                                }
                            });
                        },
                        Icon { name: "pin", size: 15.0 }
                        if pinned { " Levesz" } else { " Kitűz" }
                    }
                }
                if !msg.replies.is_empty() {
                    div { class: "ds-replies",
                        for r in msg.replies.clone() {
                            Message {
                                key: "{r.id}",
                                msg: r,
                                closed,
                                onreply: move |id| onreply.call(id),
                                onchanged: move |_| onchanged.call(()),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PollBuilder(thread_id: i64, oncancel: EventHandler<()>, oncreated: EventHandler<()>) -> Element {
    let mut question = use_signal(String::new);
    let mut ptype = use_signal(|| "date");
    let mut mode = use_signal(|| "single");
    let mut opts = use_signal(|| vec![String::new(), String::new()]);
    let mut error = use_signal(String::new);
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
            ErrorNote { message: error() }
            input {
                class: "bg-input",
                placeholder: "Kérdés — pl. Melyik hétvégén?",
                value: "{question}",
                oninput: move |e| question.set(e.value()),
            }
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
                                value: "{opts.read()[i]}",
                                oninput: move |e| {
                                    opts.write()[i] = e.value();
                                },
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
                    onclick: move |_| {
                        let q = question();
                        let options = opts.read().clone();
                        let pt = ptype().to_string();
                        let md = mode().to_string();
                        spawn(async move {
                            match api::discussions::create_poll(thread_id, q, pt, md, options).await {
                                Ok(_) => oncreated.call(()),
                                Err(e) => error.set(clean_err(&e.to_string())),
                            }
                        });
                    },
                    Icon { name: "check", size: 16.0 }
                    " Szavazás indítása"
                }
                button { class: "bg-btn ghost", onclick: move |_| oncancel.call(()), "Mégse" }
            }
        }
    }
}

#[component]
fn NewThreadView() -> Element {
    let state = use_context::<AppState>();
    let mut title = use_signal(String::new);
    let mut body = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut busy = use_signal(|| false);
    rsx! {
        div { class: "ds-thread-view bg-fade",
            div { class: "ds-thead",
                div { style: "flex: 1; min-width: 0;",
                    h2 { "Új beszélgetés" }
                    div { class: "ds-tmeta",
                        span { class: "bg-chip",
                            Icon { name: "chat", size: 13.0 }
                            " Általános téma"
                        }
                    }
                }
                button {
                    class: "bg-iconbtn",
                    onclick: move |_| {
                        let mut c = state.disc_creating;
                        c.set(false);
                    },
                    Icon { name: "x", size: 18.0 }
                }
            }
            div { class: "ds-scroll",
                div { class: "bg-card", style: "padding: 20px; display: flex; flex-direction: column; gap: 14px; max-width: 560px;",
                    ErrorNote { message: error() }
                    div { class: "bg-field",
                        label { "Téma címe" }
                        input {
                            class: "bg-input",
                            placeholder: "Miről beszélgessünk?",
                            value: "{title}",
                            oninput: move |e| title.set(e.value()),
                        }
                    }
                    div { class: "bg-field",
                        label { "Első üzenet" }
                        textarea {
                            class: "bg-input",
                            rows: 4,
                            style: "resize: none;",
                            placeholder: "Írd le, miről van szó…",
                            value: "{body}",
                            oninput: move |e| body.set(e.value()),
                        }
                    }
                    button {
                        class: "bg-btn",
                        style: "justify-content: center;",
                        disabled: busy(),
                        onclick: move |_| {
                            if busy() {
                                return;
                            }
                            busy.set(true);
                            let mut state = state;
                            spawn(async move {
                                match api::discussions::create_thread(title(), body()).await {
                                    Ok(id) => {
                                        state.threads.restart();
                                        let mut c = state.disc_creating;
                                        c.set(false);
                                        let mut a = state.active_thread;
                                        a.set(Some(id));
                                    }
                                    Err(e) => error.set(clean_err(&e.to_string())),
                                }
                                busy.set(false);
                            });
                        },
                        Icon { name: "chat", size: 16.0 }
                        " Beszélgetés indítása"
                    }
                }
            }
        }
    }
}

#[component]
fn ThreadView(thread_id: i64) -> Element {
    let state = use_context::<AppState>();
    let device_mobile = (state.is_mobile)();
    let mut menu = use_signal(|| false);
    let mut compose_poll = use_signal(|| false);
    let mut draft = use_signal(String::new);
    let mut reply_to = use_signal(|| None::<i64>);
    let mut error = use_signal(String::new);

    let mut detail = use_resource(use_reactive!(|thread_id| async move {
        api::discussions::get_thread(thread_id).await.ok()
    }));

    let Some(Some(ThreadDetailDto { thread, messages })) = detail.value()() else {
        return rsx! {
            div { class: "ds-thread-view", style: "display: grid; place-items: center; color: var(--ink-3);", "Betöltés…" }
        };
    };

    let me = state.me();
    let author = state.user(thread.author);
    let (kind_label, kind_cls) = match thread.kind.as_str() {
        "reservation" => ("Foglalás", "closed"),
        "task" => ("Feladat", "reed"),
        _ => ("Általános", ""),
    };
    let closed = thread.closed;
    let can_delete = me.id == thread.author || me.approver;
    let link_id = thread.link_id;
    let kind = thread.kind.clone();

    let send = move || {
        let body = draft();
        if body.trim().is_empty() {
            return;
        }
        let parent = reply_to();
        let mut state = state;
        spawn(async move {
            match api::discussions::post_message(thread_id, parent, body).await {
                Ok(_) => {
                    draft.set(String::new());
                    reply_to.set(None);
                    detail.restart();
                    state.threads.restart();
                }
                Err(e) => error.set(clean_err(&e.to_string())),
            }
        });
    };

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
                            Avatar { user: author.clone(), size: "sm" }
                            " {author.name}"
                        }
                        if let Some(lid) = link_id {
                            {
                                let kind2 = kind.clone();
                                rsx! {
                                    button {
                                        class: "bg-chip linkchip {kind_cls}",
                                        title: if kind == "task" { "Ugrás a feladatra" } else { "Ugrás a foglalásra" },
                                        onclick: move |_| state.open_link(&kind2, lid),
                                        Icon { name: "link", size: 13.0 }
                                        span { class: "lc-txt", " {kind_label}: {thread.link_label} " }
                                        Icon { name: "chevright", size: 13.0 }
                                    }
                                }
                            }
                        }
                        if closed {
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
                            if me.approver {
                                button {
                                    class: "ds-menu-item",
                                    onclick: move |_| {
                                        menu.set(false);
                                        let mut state = state;
                                        spawn(async move {
                                            if api::discussions::set_thread_closed(thread_id, !closed).await.is_ok() {
                                                detail.restart();
                                                state.threads.restart();
                                            }
                                        });
                                    },
                                    Icon { name: "lock", size: 16.0 }
                                    if closed { " Beszélgetés újranyitása" } else { " Beszélgetés lezárása" }
                                }
                            }
                            if can_delete {
                                div { class: "ds-menu-sep" }
                                button {
                                    class: "ds-menu-item danger",
                                    onclick: move |_| {
                                        menu.set(false);
                                        let mut state = state;
                                        spawn(async move {
                                            if api::discussions::delete_thread(thread_id).await.is_ok() {
                                                let mut a = state.active_thread;
                                                a.set(None);
                                                state.threads.restart();
                                            }
                                        });
                                    },
                                    Icon { name: "x", size: 16.0 }
                                    " Törlés"
                                }
                            }
                        }
                    }
                }
            }
            div { class: "ds-scroll",
                if messages.is_empty() {
                    div { class: "ds-sys",
                        Icon { name: "chat", size: 14.0 }
                        " Még nincs üzenet — kezdd te a beszélgetést!"
                    }
                }
                for m in messages {
                    Message {
                        key: "{m.id}",
                        msg: m,
                        closed,
                        onreply: move |id| reply_to.set(Some(id)),
                        onchanged: move |_| detail.restart(),
                    }
                }
            }
            if closed {
                div {
                    class: "ds-composer",
                    style: "justify-content: center; color: var(--ink-3); font-size: 13.5px; font-weight: 600; gap: 8px;",
                    Icon { name: "lock", size: 15.0 }
                    " Ezt a beszélgetést lezárták — csak engedélyező nyithatja meg újra."
                }
            } else if compose_poll() {
                PollBuilder {
                    thread_id,
                    oncancel: move |_| compose_poll.set(false),
                    oncreated: move |_| {
                        compose_poll.set(false);
                        detail.restart();
                    },
                }
            } else {
                div { style: "display: flex; flex-direction: column;",
                    ErrorNote { message: error() }
                    if reply_to().is_some() {
                        div { style: "display: flex; align-items: center; gap: 8px; padding: 8px 24px 0; color: var(--ink-2); font-size: 12.5px; font-weight: 600;",
                            Icon { name: "reply", size: 14.0 }
                            " Válasz egy üzenetre"
                            button {
                                class: "bg-iconbtn",
                                style: "width: 24px; height: 24px;",
                                onclick: move |_| reply_to.set(None),
                                Icon { name: "x", size: 12.0 }
                            }
                        }
                    }
                    div { class: "ds-composer",
                        Avatar { user: me.clone() }
                        textarea {
                            class: "bg-input ds-input",
                            placeholder: "Írj egy üzenetet…",
                            value: "{draft}",
                            oninput: move |e| draft.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter && !e.modifiers().shift() {
                                    e.prevent_default();
                                    send();
                                }
                            },
                        }
                        button {
                            class: "bg-iconbtn ds-pollbtn",
                            title: "Szavazás létrehozása",
                            onclick: move |_| compose_poll.set(true),
                            Icon { name: "list", size: 18.0 }
                        }
                        button {
                            class: "bg-btn",
                            style: "flex-shrink: 0; width: 44px; padding: 0; justify-content: center;",
                            onclick: move |_| send(),
                            Icon { name: "send", size: 18.0, fill: true }
                        }
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
    let list = filtered(&state);
    let active = effective_thread(&state, device_mobile);
    let creating = (state.disc_creating)();
    let show_thread = device_mobile && (active.is_some() || creating);

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
                    if list.is_empty() {
                        div { class: "tk-empty",
                            Icon { name: "chat", size: 26.0, stroke: 2.0 }
                            " Még nincs beszélgetés."
                        }
                    }
                    for t in list {
                        {
                            let author = state.user(t.author);
                            let tid = t.id;
                            rsx! {
                                div {
                                    key: "{t.id}",
                                    class: if active == Some(t.id) && !device_mobile { "ds-thread active" } else { "ds-thread" },
                                    onclick: move |_| {
                                        let mut c = state.disc_creating;
                                        c.set(false);
                                        let mut a = state.active_thread;
                                        a.set(Some(tid));
                                    },
                                    div { style: "display: flex; align-items: center; gap: 8px;",
                                        Icon { name: "{kind_icon(&t.kind)}", size: 15.0, stroke: 2.0, style: "color: var(--water); flex-shrink: 0;" }
                                        span { class: "tt", style: "flex: 1;", "{t.title}" }
                                        if t.closed {
                                            Icon { name: "lock", size: 14.0, style: "color: var(--ink-3);" }
                                        }
                                    }
                                    if !t.excerpt.is_empty() {
                                        div { class: "ex", "{t.excerpt}" }
                                    }
                                    div { class: "mt",
                                        Avatar { user: author.clone(), size: "sm" }
                                        span { "{author.name}" }
                                        span { "· {time_label(t.created_at)}" }
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
                    }
                }
                if creating {
                    NewThreadView {}
                } else if let Some(id) = active {
                    ThreadView { thread_id: id }
                }
            }
        }
    }
}

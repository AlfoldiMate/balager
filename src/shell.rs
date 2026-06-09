//! App shell: sidebar, top bar, mobile navigation, tool routing.

use dioxus::prelude::*;

use crate::common::{Avatar, BalatonMark, WaterWaves};
use crate::discussions::{DiscHeaderLeft, DiscHeaderRight, DiscussionsTool};
use crate::icons::Icon;
use crate::info::InfoTool;
use crate::models::UserDto;
use crate::notifications::NotificationsPopover;
use crate::reservations::{ReservationsTool, ResHeaderLeft, ResHeaderRight};
use crate::settings::SettingsView;
use crate::state::{use_app_state, AppState};
use crate::tasks::{TasksHeaderLeft, TasksHeaderRight, TasksTool};

struct Tool {
    id: &'static str,
    label: &'static str,
    icon: &'static str,
}

static TOOLS: &[Tool] = &[
    Tool { id: "foglalasok", label: "Foglalások", icon: "calendar" },
    Tool { id: "feladatok", label: "Feladatok", icon: "tasks" },
    Tool { id: "beszelgetesek", label: "Beszélgetések", icon: "chat" },
    Tool { id: "informacio", label: "Információ", icon: "info" },
];

fn tool_title(id: &str) -> &'static str {
    match id {
        "foglalasok" => "Foglalások",
        "feladatok" => "Feladatok",
        "beszelgetesek" => "Beszélgetések",
        "informacio" => "Információ",
        "beallitasok" => "Beállítások",
        _ => "",
    }
}

fn tool_sub(id: &str) -> &'static str {
    match id {
        "foglalasok" => "Heti naptár · Balaton",
        "feladatok" => "Közös teendők",
        "beszelgetesek" => "Családi témák",
        "informacio" => "Tudnivalók és házirend",
        "beallitasok" => "Profil és értesítések",
        _ => "",
    }
}

/// Live badge per tool: pending reservations / my open tasks.
fn badge_of(state: &AppState, tool_id: &str) -> Option<usize> {
    let n = match tool_id {
        "foglalasok" => state
            .reservations_list()
            .iter()
            .filter(|r| r.status == "pending")
            .count(),
        "feladatok" => {
            let me = state.me().id;
            state
                .groups_list()
                .iter()
                .flat_map(|g| g.tasks.iter())
                .filter(|t| !t.done && t.assignee == Some(me))
                .count()
        }
        _ => 0,
    };
    (n > 0).then_some(n)
}

#[component]
fn Sidebar() -> Element {
    let state = use_context::<AppState>();
    let open = (state.sidebar_open)();
    let active = (state.active)();
    let me = state.me();
    rsx! {
        nav { class: if open { "bg-side open" } else { "bg-side" },
            WaterWaves { height: 230.0 }
            div { class: "brand",
                if open {
                    b { class: "brand-wordmark", "Balager" }
                } else {
                    div { class: "brand-mini", BalatonMark { width: 42.0 } }
                }
            }
            div { class: "bg-nav",
                for t in TOOLS {
                    {
                        let badge = badge_of(&state, t.id);
                        rsx! {
                            button {
                                key: "{t.id}",
                                class: if active == t.id { "bg-navitem active" } else { "bg-navitem" },
                                title: "{t.label}",
                                onclick: move |_| state.set_active_tool(t.id),
                                Icon { name: "{t.icon}", size: 21.0, stroke: 2.0 }
                                if open {
                                    span { class: "lbl", "{t.label}" }
                                }
                                if open {
                                    if let Some(b) = badge {
                                        span { class: "badge-dot", "{b}" }
                                    }
                                } else {
                                    if badge.is_some() {
                                        span { class: "nav-dot" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "side-foot",
                button {
                    class: "side-toggle",
                    title: "Menü",
                    onclick: move |_| {
                        let mut s = state.sidebar_open;
                        let v = s();
                        s.set(!v);
                    },
                    Icon { name: "panelopen", size: 20.0, stroke: 2.0 }
                    if open {
                        span { class: "lbl", "Menü összecsukása" }
                    }
                }
                button {
                    class: if active == "beallitasok" { "side-user active" } else { "side-user" },
                    title: "Beállítások",
                    onclick: move |_| state.open_settings(),
                    Avatar { user: me.clone(), size: "sm", ring: false }
                    if open {
                        div { style: "min-width: 0; text-align: left;",
                            div { style: "font-weight: 700; font-size: 13.5px; line-height: 1.1;", "{me.name}" }
                            div { style: "font-size: 11px; color: rgba(255,255,255,.65);",
                                if me.approver { "Engedélyező" } else { "Családtag" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ActiveTool() -> Element {
    let state = use_context::<AppState>();
    match (state.active)() {
        "foglalasok" => rsx! { ReservationsTool {} },
        "feladatok" => rsx! { TasksTool {} },
        "beszelgetesek" => rsx! { DiscussionsTool {} },
        "informacio" => rsx! { InfoTool {} },
        _ => rsx! { SettingsView {} },
    }
}

#[component]
pub fn Balager(user: UserDto) -> Element {
    let state = use_app_state(user);

    // Track the viewport so the layout switches between desktop and mobile.
    use_future(move || async move {
        let mut is_mobile = state.is_mobile;
        let mut eval = document::eval(
            r#"
            const mq = window.matchMedia('(max-width: 768px)');
            dioxus.send(mq.matches);
            mq.addEventListener('change', (e) => dioxus.send(e.matches));
            "#,
        );
        while let Ok(v) = eval.recv::<bool>().await {
            is_mobile.set(v);
        }
    });

    let active = (state.active)();
    let notif = (state.notif_open)();
    let unread = state.notifs_list().iter().filter(|n| n.unread).count();
    let me = state.me();

    if (state.is_mobile)() {
        return rsx! {
            div { class: "bg-app is-mobile",
                div { class: "bg-mtop",
                    WaterWaves { height: 70.0 }
                    div { style: "flex: 1; min-width: 0;",
                        h1 { "{tool_title(active)}" }
                        div { class: "sub", "{tool_sub(active)}" }
                    }
                    button {
                        class: "mbtn",
                        onclick: move |_| {
                            let mut n = state.notif_open;
                            n.set(true);
                        },
                        Icon { name: "bell", size: 19.0 }
                        if unread > 0 {
                            span { class: "dot" }
                        }
                    }
                    button {
                        class: "mbtn",
                        style: "padding: 0; overflow: hidden;",
                        onclick: move |_| state.open_settings(),
                        Avatar { user: me.clone(), ring: false }
                    }
                }
                div { class: "bg-body", ActiveTool {} }
                div { class: "bg-mnav",
                    for t in TOOLS {
                        button {
                            key: "{t.id}",
                            class: if active == t.id { "on" } else { "" },
                            onclick: move |_| state.set_active_tool(t.id),
                            span { class: "mi", Icon { name: "{t.icon}", size: 21.0, stroke: 2.0 } }
                            "{t.label}"
                        }
                    }
                }
                if notif {
                    NotificationsPopover {}
                }
            }
        };
    }

    rsx! {
        div { class: "bg-app is-desktop",
            Sidebar {}
            div { class: "bg-main",
                div { class: "bg-top",
                    if active == "beallitasok" {
                        button {
                            class: "bg-btn ghost bg-top-back",
                            onclick: move |_| state.set_active_tool((state.back_to)()),
                            Icon { name: "chevleft", size: 16.0, stroke: 2.4 }
                            " Vissza"
                        }
                    } else if active == "informacio" {
                        div { class: "title-wrap",
                            h1 { "{tool_title(active)}" }
                            div { class: "sub", "{tool_sub(active)}" }
                        }
                    }
                    div { class: "bg-top-actions l",
                        match active {
                            "foglalasok" => rsx! { ResHeaderLeft {} },
                            "feladatok" => rsx! { TasksHeaderLeft {} },
                            "beszelgetesek" => rsx! { DiscHeaderLeft {} },
                            _ => rsx! {},
                        }
                    }
                    div { class: "bg-top-spacer" }
                    div { class: "bg-top-actions r",
                        match active {
                            "foglalasok" => rsx! { ResHeaderRight {} },
                            "feladatok" => rsx! { TasksHeaderRight {} },
                            "beszelgetesek" => rsx! { DiscHeaderRight {} },
                            _ => rsx! {},
                        }
                    }
                    button {
                        class: if notif { "bg-iconbtn on" } else { "bg-iconbtn" },
                        title: "Értesítések",
                        onclick: move |_| {
                            let mut n = state.notif_open;
                            let v = n();
                            n.set(!v);
                        },
                        Icon { name: "bell", size: 19.0 }
                        if unread > 0 {
                            span { class: "dot" }
                        }
                    }
                }
                div { class: "bg-body", ActiveTool {} }
            }
            if notif {
                NotificationsPopover {}
            }
        }
    }
}

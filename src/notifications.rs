//! Notifications: bell popover (desktop) / bottom sheet (mobile).

use dioxus::prelude::*;

use crate::data::{user, NOTIFS};
use crate::icons::Icon;
use crate::state::AppState;

#[component]
pub fn NotificationsPopover() -> Element {
    let state = use_context::<AppState>();
    let unread = NOTIFS.iter().filter(|n| n.unread).count();
    let close = move |_| {
        let mut n = state.notif_open;
        n.set(false);
    };

    rsx! {
        div { class: "bg-notif-scrim", onclick: close }
        div { class: "bg-notif",
            div { class: "nh",
                h3 { "Értesítések" }
                if unread > 0 {
                    span { class: "npill", "{unread} új" }
                }
                button {
                    class: "bg-iconbtn",
                    style: "width: 32px; height: 32px; border: none; background: transparent;",
                    onclick: close,
                    Icon { name: "x", size: 17.0 }
                }
            }
            div { class: "nlist",
                for n in NOTIFS {
                    div { key: "{n.id}", class: if n.unread { "notif-row unread" } else { "notif-row" },
                        div { class: "notif-ic {n.tone}",
                            Icon { name: "{n.icon}", size: 18.0, stroke: 2.2 }
                        }
                        div { style: "flex: 1; min-width: 0;",
                            div { class: "ntext",
                                if n.who != "system" {
                                    b { "{user(n.who).name} " }
                                }
                                "{n.text}"
                            }
                            div { class: "ntime", "{n.time}" }
                        }
                        if n.unread {
                            span { class: "unreaddot" }
                        }
                    }
                }
            }
            div { class: "nfoot",
                button { "Összes megjelölése olvasottként" }
            }
        }
    }
}

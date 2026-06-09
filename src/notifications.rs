//! Notifications: bell popover (desktop) / bottom sheet (mobile) — live data.

use dioxus::prelude::*;

use crate::api;
use crate::icons::Icon;
use crate::models::time_label;
use crate::state::AppState;

#[component]
pub fn NotificationsPopover() -> Element {
    let state = use_context::<AppState>();
    let notifs = state.notifs_list();
    let unread = notifs.iter().filter(|n| n.unread).count();
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
                if notifs.is_empty() {
                    div { style: "padding: 28px 16px; text-align: center; color: var(--ink-3); font-size: 13.5px;",
                        "Még nincs értesítésed."
                    }
                }
                for n in notifs {
                    {
                        let notif_id = n.id;
                        let link = n.link_kind.clone().zip(n.link_id);
                        rsx! {
                            div {
                                key: "{n.id}",
                                class: if n.unread { "notif-row unread" } else { "notif-row" },
                                onclick: move |_| {
                                    let mut state = state;
                                    let link = link.clone();
                                    spawn(async move {
                                        let _ = api::notifications::mark_notification_read(notif_id).await;
                                        state.notifs.restart();
                                        if let Some((kind, id)) = link {
                                            state.open_link(&kind, id);
                                        }
                                    });
                                },
                                div { class: "notif-ic {n.tone}",
                                    Icon { name: "{n.icon}", size: 18.0, stroke: 2.2 }
                                }
                                div { style: "flex: 1; min-width: 0;",
                                    div { class: "ntext", "{n.text}" }
                                    div { class: "ntime", "{time_label(n.created_at)}" }
                                }
                                if n.unread {
                                    span { class: "unreaddot" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "nfoot",
                button {
                    onclick: move |_| {
                        let mut state = state;
                        spawn(async move {
                            if api::notifications::mark_all_notifications_read().await.is_ok() {
                                state.notifs.restart();
                            }
                        });
                    },
                    "Összes megjelölése olvasottként"
                }
            }
        }
    }
}

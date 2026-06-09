//! Settings: profile + notification preferences (e-mail + PWA push).

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::common::{Avatar, Switch};
use crate::data::{user, ME, NOTIF_GROUPS};
use crate::icons::Icon;
use crate::state::AppState;

#[component]
pub fn SettingsView() -> Element {
    let state = use_context::<AppState>();
    let me = user(ME);
    let mut prefs = use_signal(|| {
        NOTIF_GROUPS
            .iter()
            .flat_map(|g| g.rows.iter().map(|r| (r.id, (r.email, r.push))))
            .collect::<HashMap<&'static str, (bool, bool)>>()
    });
    let mut profile_open = use_signal(|| false);
    let prefs_now = prefs.read().clone();

    rsx! {
        div { class: "bg-content bg-fade",
            div { class: "set-wrap",
                button {
                    class: "set-back",
                    onclick: move |_| state.set_active_tool((state.back_to)()),
                    Icon { name: "chevleft", size: 16.0 }
                    " Vissza"
                }

                div { class: "bg-card set-profcard",
                    button {
                        class: "set-profhead",
                        onclick: move |_| {
                            let v = profile_open();
                            profile_open.set(!v);
                        },
                        Avatar { id: "{ME}", size: "lg", ring: false }
                        div { style: "flex: 1; text-align: left;",
                            div { class: "nm", "{me.name}" }
                            div { class: "em", "csaba@balager.hu · Családtag" }
                        }
                        Icon {
                            name: "chevdown",
                            size: 18.0,
                            style: if profile_open() {
                                "color: var(--ink-3); transform: rotate(180deg); transition: transform .18s;"
                            } else {
                                "color: var(--ink-3); transition: transform .18s;"
                            },
                        }
                    }
                    if profile_open() {
                        div { class: "set-profedit",
                            div { class: "set-avedit",
                                div { class: "set-avwrap",
                                    Avatar { id: "{ME}", size: "lg", ring: false }
                                    span { class: "set-avcam", Icon { name: "camera", size: 14.0, stroke: 2.0 } }
                                }
                                div {
                                    button { class: "bg-btn ghost sm",
                                        Icon { name: "camera", size: 15.0 }
                                        " Profilkép módosítása"
                                    }
                                    p { style: "font-size: 12px; color: var(--ink-3); margin-top: 6px;", "JPG vagy PNG, max. 2 MB." }
                                }
                            }
                            div { class: "bg-field",
                                label { "Név" }
                                input { class: "bg-input", initial_value: "Csaba" }
                            }
                            div { class: "bg-field",
                                label { "E-mail cím" }
                                input { class: "bg-input", r#type: "email", initial_value: "csaba@balager.hu" }
                            }
                            div { class: "set-pwgrid",
                                div { class: "bg-field",
                                    label { "Jelenlegi jelszó" }
                                    input { class: "bg-input", r#type: "password", placeholder: "Jelenlegi jelszó…" }
                                }
                                div { class: "bg-field",
                                    label { "Új jelszó" }
                                    input { class: "bg-input", r#type: "password", placeholder: "Új jelszó…" }
                                }
                            }
                            div { style: "display: flex; gap: 10px;",
                                button { class: "bg-btn", onclick: move |_| profile_open.set(false),
                                    Icon { name: "check", size: 16.0 }
                                    " Mentés"
                                }
                                button { class: "bg-btn ghost", onclick: move |_| profile_open.set(false), "Mégse" }
                            }
                        }
                    }
                }

                div { class: "set-pwa",
                    div { class: "pi", Icon { name: "phone", size: 20.0 } }
                    div { class: "pt",
                        b { "Push értesítések ezen az eszközön" }
                        p { "A Balager telepíthető a kezdőképernyőre (PWA). Az iOS push engedélyezve." }
                    }
                    Switch { on: true, onclick: move |_| {} }
                }

                div {
                    div { style: "font-size: 13px; font-weight: 700; color: var(--ink-3); text-transform: uppercase; letter-spacing: .04em; padding: 4px 4px 2px;",
                        "Értesítési beállítások"
                    }
                    div { class: "set-colhead",
                        span {
                            Icon { name: "mail", size: 13.0 }
                            " E-mail"
                        }
                        span {
                            Icon { name: "bell", size: 13.0 }
                            " Push"
                        }
                    }
                    div { style: "display: flex; flex-direction: column; gap: 12px;",
                        for g in NOTIF_GROUPS {
                            div { class: "bg-card set-card", key: "{g.id}",
                                div { class: "set-grouphead",
                                    div { class: "gi", Icon { name: "{g.icon}", size: 17.0, stroke: 2.0 } }
                                    h4 { "{g.label}" }
                                }
                                for r in g.rows {
                                    div { class: "set-row", key: "{r.id}",
                                        div { class: "rl",
                                            div { class: "t", "{r.label}" }
                                            if !r.sub.is_empty() {
                                                div { class: "s", "{r.sub}" }
                                            }
                                        }
                                        div { class: "toggles",
                                            div { class: "tg",
                                                Switch {
                                                    on: prefs_now.get(r.id).map(|p| p.0).unwrap_or(false),
                                                    onclick: move |_| {
                                                        let mut p = prefs.write();
                                                        if let Some(v) = p.get_mut(r.id) {
                                                            v.0 = !v.0;
                                                        }
                                                    },
                                                }
                                            }
                                            div { class: "tg",
                                                Switch {
                                                    on: prefs_now.get(r.id).map(|p| p.1).unwrap_or(false),
                                                    onclick: move |_| {
                                                        let mut p = prefs.write();
                                                        if let Some(v) = p.get_mut(r.id) {
                                                            v.1 = !v.1;
                                                        }
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                button { class: "bg-btn ghost", style: "align-self: flex-start;",
                    Icon { name: "logout", size: 16.0 }
                    " Kijelentkezés"
                }
            }
        }
    }
}

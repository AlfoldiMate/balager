//! Settings: profile, password, notification preferences, user administration.

use dioxus::prelude::*;

use crate::api;
use crate::app::SessionHandle;
use crate::common::{Avatar, ErrorNote, Switch};
use crate::icons::Icon;
use crate::login::clean_err;
use crate::models::{PrefDto, UserDto, PREF_GROUPS};
use crate::state::AppState;

const AVATAR_COLORS: [&str; 8] = [
    "#3f8aa3", "#c47b4a", "#5a9b7c", "#a86fa0", "#cf9d3a", "#6b86b3", "#b1534a", "#356b9b",
];

#[component]
pub fn SettingsView() -> Element {
    let state = use_context::<AppState>();
    let me = state.me();
    rsx! {
        div { class: "bg-content bg-fade",
            div { class: "set-wrap",
                button {
                    class: "set-back",
                    onclick: move |_| state.set_active_tool((state.back_to)()),
                    Icon { name: "chevleft", size: 16.0 }
                    " Vissza"
                }
                ProfileCard {}
                div { class: "set-pwa",
                    div { class: "pi", Icon { name: "phone", size: 20.0 } }
                    div { class: "pt",
                        b { "Telepítés a kezdőképernyőre (PWA)" }
                        p { "iPhone-on: Megosztás → „Hozzáadás a kezdőképernyőhöz”. Push értesítés később érkezik; addig e-mailben értesítünk." }
                    }
                }
                PrefsSection {}
                if me.approver {
                    AdminSection {}
                }
                button {
                    class: "bg-btn ghost",
                    style: "align-self: flex-start;",
                    onclick: move |_| {
                        spawn(async move {
                            let _ = api::auth::logout().await;
                            // Full reload clears all client state.
                            document::eval("window.location.reload()");
                        });
                    },
                    Icon { name: "logout", size: 16.0 }
                    " Kijelentkezés"
                }
            }
        }
    }
}

#[component]
fn ProfileCard() -> Element {
    let state = use_context::<AppState>();
    let session = use_context::<SessionHandle>();
    let me = state.me();
    let mut open = use_signal(|| false);
    let mut name = use_signal(|| me.name.clone());
    let mut email = use_signal(|| me.email.clone());
    let mut color = use_signal(|| me.color.clone());
    let mut current_pw = use_signal(String::new);
    let mut new_pw = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut info = use_signal(String::new);

    let save = move |_| {
        let mut state = state;
        spawn(async move {
            error.set(String::new());
            info.set(String::new());
            if let Err(e) = api::users::update_profile(name(), email(), color()).await {
                error.set(clean_err(&e.to_string()));
                return;
            }
            if !new_pw().is_empty() {
                if let Err(e) = api::users::change_password(current_pw(), new_pw()).await {
                    error.set(clean_err(&e.to_string()));
                    return;
                }
                current_pw.set(String::new());
                new_pw.set(String::new());
            }
            state.users.restart();
            let mut session_res = session.0;
            session_res.restart();
            info.set("Elmentve.".into());
        });
    };

    rsx! {
        div { class: "bg-card set-profcard",
            button {
                class: "set-profhead",
                onclick: move |_| {
                    let v = open();
                    open.set(!v);
                },
                Avatar { user: me.clone(), size: "lg", ring: false }
                div { style: "flex: 1; text-align: left;",
                    div { class: "nm", "{me.name}" }
                    div { class: "em",
                        "{me.email} · "
                        if me.approver { "Engedélyező" } else { "Családtag" }
                    }
                }
                Icon {
                    name: "chevdown",
                    size: 18.0,
                    style: if open() {
                        "color: var(--ink-3); transform: rotate(180deg); transition: transform .18s;"
                    } else {
                        "color: var(--ink-3); transition: transform .18s;"
                    },
                }
            }
            if open() {
                div { class: "set-profedit",
                    ErrorNote { message: error() }
                    if !info().is_empty() {
                        div { style: "padding: 10px 13px; border-radius: 10px; background: var(--st-open-bg); border: 1px solid var(--st-open-bd); color: var(--st-open-ink); font-size: 13px;",
                            "{info}"
                        }
                    }
                    div { class: "bg-field",
                        label { "Név" }
                        input { class: "bg-input", value: "{name}", oninput: move |e| name.set(e.value()) }
                    }
                    div { class: "bg-field",
                        label { "E-mail cím" }
                        input { class: "bg-input", r#type: "email", value: "{email}", oninput: move |e| email.set(e.value()) }
                    }
                    div { class: "bg-field",
                        label { "Szín" }
                        div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                            for c in AVATAR_COLORS {
                                button {
                                    key: "{c}",
                                    style: {
                                        let ring = if color() == c { "box-shadow: 0 0 0 3px var(--water-soft), 0 0 0 5px var(--water);" } else { "" };
                                        format!("width: 30px; height: 30px; border-radius: 50%; background: {c}; {ring}")
                                    },
                                    onclick: move |_| color.set(c.to_string()),
                                }
                            }
                        }
                    }
                    div { class: "set-pwgrid",
                        div { class: "bg-field",
                            label { "Jelenlegi jelszó" }
                            input {
                                class: "bg-input",
                                r#type: "password",
                                placeholder: "Csak jelszócseréhez",
                                value: "{current_pw}",
                                oninput: move |e| current_pw.set(e.value()),
                            }
                        }
                        div { class: "bg-field",
                            label { "Új jelszó" }
                            input {
                                class: "bg-input",
                                r#type: "password",
                                placeholder: "Új jelszó…",
                                value: "{new_pw}",
                                oninput: move |e| new_pw.set(e.value()),
                            }
                        }
                    }
                    div { style: "display: flex; gap: 10px;",
                        button { class: "bg-btn", onclick: save,
                            Icon { name: "check", size: 16.0 }
                            " Mentés"
                        }
                        button { class: "bg-btn ghost", onclick: move |_| open.set(false), "Bezárás" }
                    }
                }
            }
        }
    }
}

#[component]
fn PrefsSection() -> Element {
    let mut prefs = use_resource(|| async {
        api::notifications::get_prefs().await.unwrap_or_default()
    });
    let current: Vec<PrefDto> = prefs.value()().unwrap_or_default();

    let toggle = move |key: String, email: bool, push: bool| {
        spawn(async move {
            if api::notifications::set_pref(key, email, push).await.is_ok() {
                prefs.restart();
            }
        });
    };

    rsx! {
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
                for g in PREF_GROUPS {
                    div { class: "bg-card set-card", key: "{g.label}",
                        div { class: "set-grouphead",
                            div { class: "gi", Icon { name: "{g.icon}", size: 17.0, stroke: 2.0 } }
                            h4 { "{g.label}" }
                        }
                        for row in g.rows {
                            {
                                let pref = current
                                    .iter()
                                    .find(|p| p.key == row.key)
                                    .cloned()
                                    .unwrap_or(PrefDto { key: row.key.to_string(), email: false, push: false });
                                let (email_on, push_on) = (pref.email, pref.push);
                                rsx! {
                                    div { class: "set-row", key: "{row.key}",
                                        div { class: "rl",
                                            div { class: "t", "{row.label}" }
                                            if !row.sub.is_empty() {
                                                div { class: "s", "{row.sub}" }
                                            }
                                        }
                                        div { class: "toggles",
                                            div { class: "tg",
                                                Switch {
                                                    on: email_on,
                                                    onclick: move |_| toggle(row.key.to_string(), !email_on, push_on),
                                                }
                                            }
                                            div { class: "tg",
                                                Switch {
                                                    on: push_on,
                                                    onclick: move |_| toggle(row.key.to_string(), email_on, !push_on),
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AdminSection() -> Element {
    let state = use_context::<AppState>();
    let mut users = use_resource(|| async {
        api::users::admin_list_users().await.unwrap_or_default()
    });
    let list: Vec<UserDto> = users.value()().unwrap_or_default();
    let mut form_open = use_signal(|| false);
    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut color = use_signal(|| AVATAR_COLORS[2].to_string());
    let mut approver = use_signal(|| false);
    let mut error = use_signal(String::new);

    let mut refresh = move || {
        users.restart();
        let mut shared = state.users;
        shared.restart();
    };

    rsx! {
        div {
            div { style: "font-size: 13px; font-weight: 700; color: var(--ink-3); text-transform: uppercase; letter-spacing: .04em; padding: 4px 4px 2px;",
                "Felhasználók kezelése"
            }
            div { class: "bg-card", style: "padding: 14px 18px; display: flex; flex-direction: column; gap: 4px;",
                ErrorNote { message: error() }
                for u in list {
                    {
                        let uid = u.id;
                        let active = u.active;
                        let is_approver = u.approver;
                        rsx! {
                            div { class: "set-row", key: "{u.id}", style: "border-top: none; border-bottom: 1px solid var(--line-2); padding: 12px 0;",
                                Avatar { user: u.clone(), size: "sm" }
                                div { class: "rl",
                                    div { class: "t", style: if !active { "color: var(--ink-3); text-decoration: line-through;" } else { "" },
                                        "{u.name}"
                                        if is_approver {
                                            span { class: "bg-chip reed", style: "height: 20px; margin-left: 8px;", "Engedélyező" }
                                        }
                                    }
                                    div { class: "s", "{u.email}" }
                                }
                                button {
                                    class: "bg-btn ghost sm",
                                    title: if is_approver { "Legyen családtag" } else { "Legyen engedélyező" },
                                    onclick: move |_| {
                                        spawn(async move {
                                            match api::users::set_user_role(uid, !is_approver).await {
                                                Ok(()) => refresh(),
                                                Err(e) => error.set(clean_err(&e.to_string())),
                                            }
                                        });
                                    },
                                    Icon { name: "shield", size: 14.0 }
                                }
                                button {
                                    class: "bg-btn ghost sm",
                                    title: "Új jelszó generálása",
                                    onclick: move |_| {
                                        spawn(async move {
                                            let new_pw: String = format!("balaton{}", uid * 37 + 11);
                                            match api::users::admin_reset_password(uid, new_pw.clone()).await {
                                                Ok(()) => error.set(format!("Új jelszó {}-nak/-nek: {new_pw}", uid)),
                                                Err(e) => error.set(clean_err(&e.to_string())),
                                            }
                                        });
                                    },
                                    Icon { name: "settings", size: 14.0 }
                                }
                                Switch {
                                    on: active,
                                    onclick: move |_| {
                                        spawn(async move {
                                            match api::users::set_user_active(uid, !active).await {
                                                Ok(()) => refresh(),
                                                Err(e) => error.set(clean_err(&e.to_string())),
                                            }
                                        });
                                    },
                                }
                            }
                        }
                    }
                }
                if form_open() {
                    div { style: "display: flex; flex-direction: column; gap: 12px; padding-top: 14px;",
                        div { class: "set-pwgrid",
                            div { class: "bg-field",
                                label { "Név" }
                                input { class: "bg-input", value: "{name}", oninput: move |e| name.set(e.value()) }
                            }
                            div { class: "bg-field",
                                label { "E-mail" }
                                input { class: "bg-input", r#type: "email", value: "{email}", oninput: move |e| email.set(e.value()) }
                            }
                        }
                        div { class: "set-pwgrid",
                            div { class: "bg-field",
                                label { "Kezdeti jelszó" }
                                input { class: "bg-input", value: "{password}", oninput: move |e| password.set(e.value()) }
                            }
                            div { class: "bg-field",
                                label { "Szín" }
                                div { style: "display: flex; gap: 6px; flex-wrap: wrap; padding-top: 8px;",
                                    for c in AVATAR_COLORS {
                                        button {
                                            key: "{c}",
                                            style: {
                                                let ring = if color() == c { "box-shadow: 0 0 0 2px var(--surface), 0 0 0 4px var(--water);" } else { "" };
                                                format!("width: 24px; height: 24px; border-radius: 50%; background: {c}; {ring}")
                                            },
                                            onclick: move |_| color.set(c.to_string()),
                                        }
                                    }
                                }
                            }
                        }
                        div { style: "display: flex; align-items: center; gap: 10px;",
                            Switch { on: approver(), onclick: move |_| {
                                let v = approver();
                                approver.set(!v);
                            } }
                            span { style: "font-size: 13.5px; font-weight: 600;", "Engedélyező jogosultság" }
                        }
                        div { style: "display: flex; gap: 10px;",
                            button {
                                class: "bg-btn",
                                onclick: move |_| {
                                    spawn(async move {
                                        match api::users::create_user(name(), email(), password(), color(), approver()).await {
                                            Ok(_) => {
                                                name.set(String::new());
                                                email.set(String::new());
                                                password.set(String::new());
                                                approver.set(false);
                                                form_open.set(false);
                                                error.set(String::new());
                                                refresh();
                                            }
                                            Err(e) => error.set(clean_err(&e.to_string())),
                                        }
                                    });
                                },
                                Icon { name: "check", size: 16.0 }
                                " Felhasználó létrehozása"
                            }
                            button { class: "bg-btn ghost", onclick: move |_| form_open.set(false), "Mégse" }
                        }
                    }
                } else {
                    button {
                        class: "bg-btn ghost sm",
                        style: "align-self: flex-start; margin-top: 10px;",
                        onclick: move |_| form_open.set(true),
                        Icon { name: "plus", size: 14.0 }
                        " Új felhasználó"
                    }
                }
            }
        }
    }
}

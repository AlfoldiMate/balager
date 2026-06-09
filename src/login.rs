//! Login screen — email + password, Balaton mood.

use dioxus::prelude::*;

use crate::api;
use crate::app::SessionHandle;
use crate::common::{BalatonMark, WaterWaves};
use crate::icons::Icon;

#[component]
pub fn LoginScreen() -> Element {
    let session = use_context::<SessionHandle>();
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut busy = use_signal(|| false);

    let mut submit = move || {
        if busy() {
            return;
        }
        error.set(String::new());
        busy.set(true);
        spawn(async move {
            match api::auth::login(email(), password()).await {
                Ok(_) => {
                    let mut me = session.0;
                    me.restart();
                }
                Err(e) => error.set(clean_err(&e.to_string())),
            }
            busy.set(false);
        });
    };

    rsx! {
        div {
            class: "bg-app",
            style: "align-items: center; justify-content: center; background: linear-gradient(165deg, var(--water), var(--water-deep)); position: relative;",
            WaterWaves { height: 300.0 }
            div {
                class: "bg-card bg-fade",
                style: "position: relative; z-index: 1; width: min(380px, calc(100vw - 40px)); padding: 34px 30px 30px; display: flex; flex-direction: column; gap: 16px;",
                div { style: "display: flex; flex-direction: column; align-items: center; gap: 10px; margin-bottom: 6px;",
                    div { style: "width: 64px; height: 64px; border-radius: 18px; background: linear-gradient(150deg, var(--water-deep), var(--water)); display: grid; place-items: center; box-shadow: var(--shadow-md);",
                        BalatonMark { width: 46.0 }
                    }
                    div { style: "font-family: 'Fredoka', sans-serif; font-size: 34px; font-weight: 600; color: var(--ink);", "Balager" }
                    div { style: "font-size: 13px; color: var(--ink-3); text-align: center;", "Családi nyaraló-kezelő a Balatonra" }
                }
                div { class: "bg-field",
                    label { "E-mail cím" }
                    input {
                        class: "bg-input",
                        r#type: "email",
                        placeholder: "nev@example.hu",
                        value: "{email}",
                        autofocus: true,
                        oninput: move |e| email.set(e.value()),
                        onkeydown: move |e| {
                            if e.key() == Key::Enter {
                                submit();
                            }
                        },
                    }
                }
                div { class: "bg-field",
                    label { "Jelszó" }
                    input {
                        class: "bg-input",
                        r#type: "password",
                        placeholder: "••••••••",
                        value: "{password}",
                        oninput: move |e| password.set(e.value()),
                        onkeydown: move |e| {
                            if e.key() == Key::Enter {
                                submit();
                            }
                        },
                    }
                }
                if !error().is_empty() {
                    div { style: "padding: 10px 13px; border-radius: 10px; background: var(--st-reject-bg); border: 1px solid var(--st-reject-bd); color: var(--st-reject-ink); font-size: 13.5px;",
                        "{error}"
                    }
                }
                button {
                    class: "bg-btn",
                    style: "justify-content: center; height: 44px;",
                    disabled: busy(),
                    onclick: move |_| submit(),
                    Icon { name: "waves", size: 17.0 }
                    if busy() { " Bejelentkezés…" } else { " Bejelentkezés" }
                }
                div { style: "font-size: 12px; color: var(--ink-3); text-align: center;",
                    "Fiókot az engedélyezők tudnak létrehozni."
                }
            }
        }
    }
}

/// Strip the server-fn error prefix so users see only the Hungarian message.
pub fn clean_err(raw: &str) -> String {
    raw.rsplit("error running server function:")
        .next()
        .unwrap_or(raw)
        .trim()
        .to_string()
}

//! Root component: document head, session gate (login ⇄ shell).

use dioxus::prelude::*;

use crate::api;
use crate::login::LoginScreen;
use crate::models::UserDto;

static CSS: Asset = asset!("/assets/styles.css");
static MANIFEST: Asset = asset!("/assets/manifest.json");
static ICON: Asset = asset!("/assets/icon.png");

/// Handle for re-checking the session (after login/logout).
#[derive(Clone, Copy)]
pub struct SessionHandle(pub Resource<Option<UserDto>>);

#[component]
pub fn App() -> Element {
    rsx! {
        document::Title { "Balager — Balatoni nyaraló-kezelő" }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0, viewport-fit=cover" }
        document::Meta { name: "theme-color", content: "#28547a" }
        document::Meta { name: "mobile-web-app-capable", content: "yes" }
        document::Meta { name: "apple-mobile-web-app-capable", content: "yes" }
        document::Meta { name: "apple-mobile-web-app-status-bar-style", content: "black-translucent" }
        document::Meta { name: "apple-mobile-web-app-title", content: "Balager" }
        document::Stylesheet { href: CSS }
        document::Link { rel: "manifest", href: MANIFEST }
        document::Link { rel: "apple-touch-icon", href: ICON }
        SessionGate {}
    }
}

#[component]
fn SessionGate() -> Element {
    let session = use_resource(|| async { api::auth::me().await.ok().flatten() });
    use_context_provider(|| SessionHandle(session));

    match session.value()() {
        None => rsx! {
            div { style: "height: 100%; display: grid; place-items: center; background: var(--bg); color: var(--ink-3); font-family: 'Hanken Grotesk', sans-serif;",
                "Betöltés…"
            }
        },
        Some(None) => rsx! { LoginScreen {} },
        Some(Some(user)) => rsx! {
            crate::shell::Balager { user }
        },
    }
}

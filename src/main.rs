//! Balager — family lake-house manager (reservations, tasks, discussions).
//! UI implemented from the Claude Design handoff bundle in design/design-handoff.

mod common;
mod data;
mod discussions;
mod icons;
mod info;
mod notifications;
mod reservations;
mod settings;
mod shell;
mod state;
mod tasks;

use dioxus::prelude::*;

static CSS: Asset = asset!("/assets/styles.css");
static MANIFEST: Asset = asset!("/assets/manifest.json");
static ICON: Asset = asset!("/assets/icon.png");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
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
        shell::Balager {}
    }
}

//! Balager — family lake-house manager (reservations, tasks, discussions).
//! Fullstack Dioxus app: WASM client + axum server + SQLite.

mod api;
mod app;
#[cfg(feature = "server")]
mod backend;
mod common;
mod discussions;
mod icons;
mod info;
mod login;
mod models;
mod notifications;
mod reservations;
mod settings;
mod shell;
mod state;
mod tasks;

fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        backend::db::init().await.expect("database initialisation failed");
        Ok(dioxus::server::router(app::App))
    });

    #[cfg(not(feature = "server"))]
    dioxus::launch(app::App);
}

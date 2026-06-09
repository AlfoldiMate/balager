//! Balager — family lake-house manager (reservations, tasks, discussions).
//!
//! One binary, two roles selected by cargo features:
//! - default/`server`: an axum API server hosting the `#[server]` functions,
//!   run by Vercel's Rust runtime as a Fluid function (and as a plain HTTP
//!   server on localhost:3000 for local/self-hosted use, also serving the
//!   static client from ./public).
//! - `web`: the WASM client (`dx build --platform web --no-default-features
//!   --features web`), served statically.

mod api;
mod app;
#[cfg(feature = "server")]
mod backend;
mod common;
mod discussions;
#[cfg(feature = "server")]
mod domain;
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

#[cfg(feature = "server")]
#[tokio::main]
async fn main() -> Result<(), vercel_runtime::Error> {
    use dioxus::server::{DioxusRouterExt, FullstackState, ServeConfig};
    use tower::ServiceBuilder;
    use tower_http::services::{ServeDir, ServeFile};
    use vercel_runtime::axum::VercelLayer;

    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,sqlx=warn".to_string()),
        )
        .init();

    backend::db::init()
        .await
        .map_err(|e| -> vercel_runtime::Error { format!("database init failed: {e}").into() })?;

    // Server functions under /api/*, SSR fallback for pages (which embeds the
    // hydration payload the fullstack client expects). The HTML shell is the
    // dx-generated index.html, read from DIOXUS_PUBLIC_PATH.
    let public = std::env::var("BALAGER_PUBLIC_DIR").unwrap_or_else(|_| "public".to_string());
    // The HTML shell lives outside the CDN-served static dir so that "/" is
    // always SSR-rendered (with the hydration payload) instead of the raw file.
    let shell = std::env::var("BALAGER_SHELL_DIR").unwrap_or_else(|_| "shell".to_string());
    if std::env::var("DIOXUS_PUBLIC_PATH").is_err() {
        std::env::set_var("DIOXUS_PUBLIC_PATH", &shell);
    }
    let router = axum::Router::<FullstackState>::new()
        .serve_api_application(ServeConfig::new(), app::App)
        // Static assets for local / self-hosted runs; on Vercel the CDN
        // serves these before the rewrite reaches the function.
        .nest_service("/assets", ServeDir::new(format!("{public}/assets")))
        .nest_service("/wasm", ServeDir::new(format!("{public}/wasm")))
        .route_service("/styles.css", ServeFile::new(format!("{public}/styles.css")))
        .route_service("/manifest.json", ServeFile::new(format!("{public}/manifest.json")))
        .route_service("/icon.png", ServeFile::new(format!("{public}/icon.png")));

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);
    vercel_runtime::run(app).await
}

#[cfg(all(not(feature = "server"), feature = "web"))]
fn main() {
    dioxus::launch(app::App);
}

#[cfg(all(not(feature = "server"), not(feature = "web")))]
fn main() {
    panic!("build with --features server (API) or --features web (client)");
}

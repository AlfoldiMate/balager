//! Balager — family lake-house manager (reservations, tasks, discussions).
//!
//! One binary, two roles selected by cargo features:
//! - default/`server`: an axum API server hosting the `#[server]` functions,
//!   run by Vercel's Rust runtime as a Fluid function (and as a plain HTTP
//!   server on localhost:3000 for local/self-hosted use, also serving the
//!   static client from ./public).
//! - `web`: the WASM client (`dx build --platform web --no-default-features
//!   --features web`), served statically.
//!
//! The binary entry lives in `api/main.rs` (Vercel requires custom functions
//! to be inside the `api/` directory); it delegates to [`server_main`] /
//! [`client_main`].

pub mod api;
pub mod app;
#[cfg(feature = "server")]
pub mod backend;
pub mod common;
pub mod discussions;
#[cfg(feature = "server")]
pub mod domain;
pub mod icons;
pub mod info;
pub mod login;
pub mod models;
pub mod notifications;
pub mod reservations;
pub mod settings;
pub mod shell;
pub mod state;
pub mod tasks;

/// Entry point of the server: Vercel Fluid function / local HTTP server.
#[cfg(feature = "server")]
pub async fn server_main() -> Result<(), vercel_runtime::Error> {
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
    // The SSR head injects a loader <script> whose hashed name is derived
    // from the *server* build, which differs from the committed client
    // bundle. Alias any missing main-dxh*.js / main_bg-dxh*.wasm request to
    // the committed loader/wasm (content-identical).
    let assets_dir = format!("{public}/assets");
    let alias_shell = shell.clone();
    let asset_alias = axum::routing::get(move |uri: axum::http::Uri| {
        let shell = alias_shell.clone();
        async move {
            use axum::response::IntoResponse;
            let name = uri.path().trim_start_matches('/').to_string();
            let (path, mime): (Option<std::path::PathBuf>, &str) =
                if name.starts_with("main-dxh") && name.ends_with(".js") {
                    (Some(format!("{shell}/loader.js").into()), "text/javascript")
                } else if name.starts_with("main_bg-dxh") && name.ends_with(".wasm") {
                    (Some(format!("{shell}/main_bg.wasm").into()), "application/wasm")
                } else {
                    (None, "")
                };
            match path.and_then(|p| std::fs::read(p).ok()) {
                Some(bytes) => (
                    [(http::header::CONTENT_TYPE, mime.to_string())],
                    bytes,
                )
                    .into_response(),
                None => http::StatusCode::NOT_FOUND.into_response(),
            }
        }
    });
    let router = axum::Router::<FullstackState>::new()
        .serve_api_application(ServeConfig::new(), app::App)
        // Static assets for local / self-hosted runs; on Vercel the CDN
        // serves these before the rewrite reaches the function.
        .nest_service("/assets", ServeDir::new(assets_dir).fallback(asset_alias))
        .nest_service("/wasm", ServeDir::new(format!("{public}/wasm")))
        .route_service("/styles.css", ServeFile::new(format!("{public}/styles.css")))
        .route_service("/manifest.json", ServeFile::new(format!("{public}/manifest.json")))
        .route_service("/icon.png", ServeFile::new(format!("{public}/icon.png")));

    // The rewrite in vercel.json appends ?__orig=$1; with that shape Vercel's
    // proxy delivers the *original* method, path and query to the function
    // (verified via runtime logs), so only the marker parameter needs to be
    // stripped before routing.
    fn strip_rewrite_marker(mut req: http::Request<axum::body::Body>) -> http::Request<axum::body::Body> {
        let Some(query) = req.uri().query() else { return req };
        if !query.contains("__orig=") {
            return req;
        }
        let rest: Vec<&str> = query
            .split('&')
            .filter(|kv| !kv.starts_with("__orig="))
            .collect();
        let mut path_and_query = req.uri().path().to_string();
        if !rest.is_empty() {
            path_and_query.push('?');
            path_and_query.push_str(&rest.join("&"));
        }
        let mut parts = req.uri().clone().into_parts();
        if let Ok(pq) = http::uri::PathAndQuery::try_from(path_and_query) {
            parts.path_and_query = Some(pq);
            if let Ok(uri) = http::Uri::from_parts(parts) {
                *req.uri_mut() = uri;
            }
        }
        req
    }

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .map_request(strip_rewrite_marker)
        .service(router);
    vercel_runtime::run(app).await
}

/// Entry point of the WASM client.
#[cfg(feature = "web")]
pub fn client_main() {
    dioxus::launch(app::App);
}

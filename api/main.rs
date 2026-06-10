//! Binary entry point. Vercel's `functions` config requires the function
//! source to live in the `api/` directory; all real code is in the `balager`
//! library crate (see `src/lib.rs`).

#[cfg(feature = "server")]
#[tokio::main]
async fn main() -> Result<(), vercel_runtime::Error> {
    balager::server_main().await
}

#[cfg(all(not(feature = "server"), feature = "web"))]
fn main() {
    balager::client_main();
}

#[cfg(all(not(feature = "server"), not(feature = "web")))]
fn main() {
    panic!("build with --features server (API) or --features web (client)");
}

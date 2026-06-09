//! Transport layer: typed `#[server]` functions shared by client and server.
//! Each endpoint authenticates the request and delegates to `crate::domain`.

pub mod auth;
pub mod discussions;
pub mod notifications;
pub mod reservations;
pub mod tasks;
pub mod users;

/// Map a domain error onto the wire; the message is already user-facing.
#[cfg(feature = "server")]
pub(crate) fn de(e: crate::domain::DomainError) -> dioxus::prelude::ServerFnError {
    dioxus::prelude::ServerFnError::new(e.to_string())
}

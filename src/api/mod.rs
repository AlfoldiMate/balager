//! Application API: typed `#[server]` functions shared by client and server.
//! The bodies run on the server only; the client gets generated HTTP stubs.

pub mod auth;
pub mod discussions;
pub mod notifications;
pub mod reservations;
pub mod tasks;
pub mod users;

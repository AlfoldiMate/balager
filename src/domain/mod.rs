//! Domain layer: all business rules, independent of transport.
//!
//! The `#[server]` endpoints in `src/api/*` are thin adapters: they
//! authenticate the request (cookie → [`Actor`]) and delegate here. Everything
//! else — authorization, validation, status derivation, propagation into
//! threads, notification fan-out — lives in these modules, so the same logic
//! can be reused from a future cron job (due-date reminders), an admin CLI, a
//! bot, or tests, without going through HTTP.
//!
//! Layering:
//! - `api/*`      transport (server functions): authn + DTO mapping
//! - `domain/*`   business rules (this module): authz + invariants
//! - `backend/*`  infrastructure: DB pool, sessions/passwords, delivery (email)

pub mod discussions;
pub mod notifications;
pub mod reservations;
pub mod tasks;
pub mod users;

use std::fmt;

/// The authenticated user performing an operation.
pub use crate::backend::auth::DbUser as Actor;

pub type DomainResult<T> = Result<T, DomainError>;

#[derive(Debug)]
pub enum DomainError {
    /// Validation failure — message is user-facing (Hungarian).
    Invalid(String),
    /// The actor is not allowed to do this.
    Forbidden(String),
    NotFound(String),
    Db(sqlx::Error),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::Invalid(m) | DomainError::Forbidden(m) | DomainError::NotFound(m) => {
                write!(f, "{m}")
            }
            DomainError::Db(e) => write!(f, "Adatbázis-hiba: {e}"),
        }
    }
}

impl std::error::Error for DomainError {}

impl From<sqlx::Error> for DomainError {
    fn from(e: sqlx::Error) -> Self {
        DomainError::Db(e)
    }
}

pub(crate) fn invalid(msg: impl Into<String>) -> DomainError {
    DomainError::Invalid(msg.into())
}

pub(crate) fn forbidden(msg: impl Into<String>) -> DomainError {
    DomainError::Forbidden(msg.into())
}

pub(crate) fn not_found(msg: impl Into<String>) -> DomainError {
    DomainError::NotFound(msg.into())
}

/// Authorization guard: only approvers may pass.
pub(crate) fn require_approver(actor: &Actor) -> DomainResult<()> {
    if actor.role != "approver" {
        return Err(forbidden("Csak engedélyező végezheti el ezt a műveletet."));
    }
    Ok(())
}

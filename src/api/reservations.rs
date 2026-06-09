//! Reservation endpoints: authentication + delegation to `domain::reservations`.

use dioxus::prelude::*;

use crate::models::{NewReservation, ReservationDto};

#[server]
pub async fn list_reservations() -> Result<Vec<ReservationDto>, ServerFnError> {
    crate::backend::auth::require_user().await?;
    crate::domain::reservations::list().await.map_err(super::de)
}

#[server]
pub async fn create_reservation(input: NewReservation) -> Result<i64, ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::reservations::create(&actor, input)
        .await
        .map_err(super::de)
}

/// Approve or reject a reservation. Rejection requires a comment.
#[server]
pub async fn decide_reservation(
    res_id: i64,
    approve: bool,
    comment: String,
) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::reservations::decide(&actor, res_id, approve, comment)
        .await
        .map_err(super::de)
}

#[server]
pub async fn set_access(res_id: i64, access: String) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::reservations::set_access(&actor, res_id, access)
        .await
        .map_err(super::de)
}

/// Join an open reservation (or leave it again).
#[server]
pub async fn set_attendance(res_id: i64, attend: bool) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::reservations::set_attendance(&actor, res_id, attend)
        .await
        .map_err(super::de)
}

/// Owner manages the attendee list.
#[server]
pub async fn set_attendee(res_id: i64, user_id: i64, attend: bool) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::reservations::set_attendee(&actor, res_id, user_id, attend)
        .await
        .map_err(super::de)
}

/// Cancel a reservation (owner, or an approver).
#[server]
pub async fn delete_reservation(res_id: i64) -> Result<(), ServerFnError> {
    let actor = crate::backend::auth::require_user().await?;
    crate::domain::reservations::delete(&actor, res_id)
        .await
        .map_err(super::de)
}

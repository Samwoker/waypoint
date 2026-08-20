use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;

pub async fn list_dlq(
    _tenant: AuthenticatedTenant,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn get_dlq_item(
    _tenant: AuthenticatedTenant,
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn retry_dlq_item(
    _tenant: AuthenticatedTenant,
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn purge_dlq_item(
    _tenant: AuthenticatedTenant,
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn retry_all_dlq(
    _tenant: AuthenticatedTenant,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

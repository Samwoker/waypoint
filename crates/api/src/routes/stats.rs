use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};

use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;

pub async fn get_tenant_stats(
    _tenant: AuthenticatedTenant,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn get_system_stats(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

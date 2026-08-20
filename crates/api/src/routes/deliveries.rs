use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListDeliveriesQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_deliveries(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Query(query): Query<ListDeliveriesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let deliveries = state
        .delivery_service
        .list_deliveries(tenant.tenant_id, query.status.as_deref(), limit, offset)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(deliveries)))
}

pub async fn get_delivery(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let delivery = state
        .delivery_service
        .get_delivery(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?
        .ok_or_else(|| ApiError(relay_core::error::CoreError::NotFound(format!("Delivery '{id}' not found"))))?;

    Ok((StatusCode::OK, Json(delivery)))
}

pub async fn list_delivery_attempts(
    _tenant: AuthenticatedTenant,
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn retry_delivery(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let delivery = state
        .delivery_service
        .retry_delivery(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(delivery)))
}

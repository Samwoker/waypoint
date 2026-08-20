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
pub struct ListDlqQuery {
    pub limit: Option<i64>,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DlqActionQuery {
    pub destination_id: Option<Uuid>,
}

pub async fn list_dlq(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Query(query): Query<ListDlqQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = query.limit.unwrap_or(20);

    let paginated = state
        .delivery_service
        .list_dlq_paginated(tenant.tenant_id, limit, query.cursor.as_deref())
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(paginated)))
}

pub async fn requeue_dlq_item(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
    Query(query): Query<DlqActionQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let destination_id = query.destination_id.ok_or_else(|| {
        ApiError(relay_core::error::CoreError::Validation(
            "Query parameter 'destination_id' is required".to_string(),
        ))
    })?;

    let delivery = state
        .delivery_service
        .requeue_dlq_item(tenant.tenant_id, event_id, destination_id)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(delivery)))
}

pub async fn discard_dlq_item(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
    Query(query): Query<DlqActionQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let destination_id = query.destination_id.ok_or_else(|| {
        ApiError(relay_core::error::CoreError::Validation(
            "Query parameter 'destination_id' is required".to_string(),
        ))
    })?;

    state
        .delivery_service
        .discard_dlq_item(tenant.tenant_id, event_id, destination_id)
        .await
        .map_err(ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_dlq_item(
    tenant: AuthenticatedTenant,
    state: State<AppState>,
    path: Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    crate::routes::deliveries::get_delivery(tenant, state, path).await
}

pub async fn retry_dlq_item(
    tenant: AuthenticatedTenant,
    state: State<AppState>,
    path: Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    crate::routes::deliveries::replay_delivery(tenant, state, path, None).await
}

pub async fn purge_dlq_item(
    _tenant: AuthenticatedTenant,
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn retry_all_dlq(
    _tenant: AuthenticatedTenant,
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

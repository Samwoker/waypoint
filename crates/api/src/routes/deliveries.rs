use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;
use domain::dto::ReplayDeliveryInput;

use crate::error::ApiError;
use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListDeliveriesQuery {
    pub destination_id: Option<Uuid>,
    pub status: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub cursor: Option<String>,
}

pub async fn list_deliveries(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Query(query): Query<ListDeliveriesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = query.limit.unwrap_or(20);

    let deliveries = state
        .delivery_service
        .list_deliveries_paginated(
            tenant.tenant_id,
            query.destination_id,
            query.status.as_deref(),
            query.from,
            query.to,
            limit,
            query.cursor.as_deref(),
        )
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
        .get_delivery_detail(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(delivery)))
}

pub async fn replay_delivery(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    input: Option<Json<ReplayDeliveryInput>>,
) -> Result<impl IntoResponse, ApiError> {
    let reset_attempt_count = input.map(|Json(i)| i.reset_attempt_count).unwrap_or(false);

    let delivery = state
        .delivery_service
        .replay_delivery(tenant.tenant_id, id, reset_attempt_count)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(delivery)))
}

pub async fn retry_delivery(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let delivery = state
        .delivery_service
        .replay_delivery(tenant.tenant_id, id, false)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(delivery)))
}

pub async fn list_delivery_attempts(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let delivery = state
        .delivery_service
        .get_delivery_detail(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(delivery.attempts)))
}

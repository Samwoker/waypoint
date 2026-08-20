use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;
use domain::dto::CreateEventInput;

use crate::error::ApiError;
use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub source_id: Option<Uuid>,
    pub status: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub cursor: Option<String>,
}

pub async fn list_events(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Query(params): Query<ListEventsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = params.limit.unwrap_or(20);

    let paginated = state
        .ingestion_service
        .list_events_paginated(
            tenant.tenant_id,
            params.source_id,
            params.status.as_deref(),
            params.from,
            params.to,
            limit,
            params.cursor.as_deref(),
        )
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(paginated)))
}

pub async fn get_event(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let event = state
        .ingestion_service
        .get_event_detail(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(event)))
}

pub async fn get_event_raw(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let raw_payload = state
        .ingestion_service
        .get_event_raw(tenant.tenant_id, id, &tenant.scope)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(raw_payload)))
}

pub async fn create_event(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Json(input): Json<CreateEventInput>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!(
        tenant_id = %tenant.tenant_id,
        event_type = %input.event_type,
        "Ingesting new event"
    );

    let event = state
        .ingestion_service
        .create_event(tenant.tenant_id, input)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::ACCEPTED, Json(event)))
}

pub async fn delete_event(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .ingestion_service
        .delete_event_compliance(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_event_deliveries(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let deliveries = state
        .ingestion_service
        .get_event_deliveries(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(deliveries)))
}

pub async fn retry_event(
    _tenant: AuthenticatedTenant,
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

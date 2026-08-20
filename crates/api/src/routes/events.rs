use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use domain::dto::CreateEventInput;

use crate::error::ApiError;
use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListPagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_events(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Query(pagination): Query<ListPagination>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = pagination.limit.unwrap_or(20);
    let offset = pagination.offset.unwrap_or(0);

    let events = state
        .ingestion_service
        .list_events(tenant.tenant_id, limit, offset)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(events)))
}

pub async fn get_event(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let event = state
        .ingestion_service
        .get_event(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?
        .ok_or_else(|| ApiError(relay_core::error::CoreError::NotFound(format!("Event '{id}' not found"))))?;

    Ok((StatusCode::OK, Json(event)))
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

pub async fn get_event_deliveries(
    _tenant: AuthenticatedTenant,
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn retry_event(
    _tenant: AuthenticatedTenant,
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use domain::dto::{CreateDestinationInput, UpdateDestinationInput};

use crate::error::ApiError;
use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListPagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_destinations(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Query(pagination): Query<ListPagination>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = pagination.limit.unwrap_or(20);
    let offset = pagination.offset.unwrap_or(0);

    let destinations = state
        .destination_service
        .list_destinations(tenant.tenant_id, limit, offset)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(destinations)))
}

pub async fn create_destination(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Json(input): Json<CreateDestinationInput>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!(
        tenant_id = %tenant.tenant_id,
        destination_name = %input.name,
        destination_url = %input.url,
        "Creating new destination"
    );

    let destination = state
        .destination_service
        .create_destination(tenant.tenant_id, input)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::CREATED, Json(destination)))
}

pub async fn get_destination(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let destination = state
        .destination_service
        .get_destination(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?
        .ok_or_else(|| ApiError(relay_core::error::CoreError::NotFound(format!("Destination '{id}' not found"))))?;

    Ok((StatusCode::OK, Json(destination)))
}

pub async fn update_destination(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateDestinationInput>,
) -> Result<impl IntoResponse, ApiError> {
    let destination = state
        .destination_service
        .update_destination(tenant.tenant_id, id, input)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(destination)))
}

pub async fn delete_destination(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .destination_service
        .delete_destination(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_destination(
    tenant: AuthenticatedTenant,
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let _ = tenant;
    Ok(StatusCode::NOT_IMPLEMENTED)
}

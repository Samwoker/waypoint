use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use domain::dto::{CreateSourceInput, UpdateSourceInput};

use crate::error::ApiError;
use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListPagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_sources(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Query(pagination): Query<ListPagination>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = pagination.limit.unwrap_or(20);
    let offset = pagination.offset.unwrap_or(0);

    let sources = state
        .source_service
        .list_sources(tenant.tenant_id, limit, offset)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(sources)))
}

pub async fn create_source(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Json(input): Json<CreateSourceInput>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!(
        tenant_id = %tenant.tenant_id,
        source_name = %input.name,
        source_slug = %input.slug,
        "Creating new event source"
    );

    let source = state
        .source_service
        .create_source(tenant.tenant_id, input)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::CREATED, Json(source)))
}

pub async fn get_source(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let source = state
        .source_service
        .get_source(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?
        .ok_or_else(|| ApiError(relay_core::error::CoreError::NotFound(format!("Source '{id}' not found"))))?;

    Ok((StatusCode::OK, Json(source)))
}

pub async fn update_source(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateSourceInput>,
) -> Result<impl IntoResponse, ApiError> {
    let source = state
        .source_service
        .update_source(tenant.tenant_id, id, input)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(source)))
}

#[derive(Debug, Deserialize, Default)]
pub struct DeleteSourceQuery {
    pub force: Option<bool>,
}

pub async fn delete_source(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<DeleteSourceQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let force = query.force.unwrap_or(false);
    state
        .source_service
        .delete_source(tenant.tenant_id, id, force)
        .await
        .map_err(ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn rotate_source_secret(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .source_service
        .rotate_source_secret(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(result)))
}

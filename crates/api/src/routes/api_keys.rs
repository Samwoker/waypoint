use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use domain::dto::CreateApiKeyInput;

use crate::error::ApiError;
use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListPagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_api_keys(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Query(pagination): Query<ListPagination>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = pagination.limit.unwrap_or(20);
    let offset = pagination.offset.unwrap_or(0);

    let keys = state
        .auth_service
        .list_api_keys(tenant.tenant_id, limit, offset)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(keys)))
}

pub async fn create_api_key(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Json(input): Json<CreateApiKeyInput>,
) -> Result<impl IntoResponse, ApiError> {
    let key = state
        .auth_service
        .create_api_key(tenant.tenant_id, input)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::CREATED, Json(key)))
}

pub async fn get_api_key(
    _tenant: AuthenticatedTenant,
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn revoke_api_key(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .auth_service
        .revoke_api_key(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::NO_CONTENT, ()))
}

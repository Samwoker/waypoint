use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use domain::dto::{CreateTenantInput, UpdateTenantInput};

use crate::error::ApiError;
use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct UsageQuery {
    pub period: Option<String>,
}

pub async fn list_tenants(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn create_tenant(
    State(_state): State<AppState>,
    Json(_input): Json<CreateTenantInput>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn get_tenant(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn update_tenant(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
    Json(_input): Json<UpdateTenantInput>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn delete_tenant(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn get_tenant_usage(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<UsageQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let usage = state
        .tenant_service
        .get_tenant_usage(tenant.tenant_id, tenant.is_admin, id, query.period.as_deref())
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(usage)))
}

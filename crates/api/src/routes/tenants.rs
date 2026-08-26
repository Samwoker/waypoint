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

#[derive(Debug, Deserialize)]
pub struct ListPagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_tenants(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Query(pagination): Query<ListPagination>,
) -> Result<impl IntoResponse, ApiError> {
    if !tenant.is_admin {
        // Return single caller tenant if not platform admin
        let caller_tenant = state
            .tenant_service
            .get_tenant(tenant.tenant_id)
            .await
            .map_err(ApiError)?
            .into_iter()
            .collect::<Vec<_>>();
        return Ok((StatusCode::OK, Json(caller_tenant)));
    }

    let limit = pagination.limit.unwrap_or(20);
    let offset = pagination.offset.unwrap_or(0);

    let tenants = state
        .tenant_service
        .list_tenants(limit, offset)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(tenants)))
}

pub async fn create_tenant(
    State(state): State<AppState>,
    Json(input): Json<CreateTenantInput>,
) -> Result<impl IntoResponse, ApiError> {
    let tenant = state
        .tenant_service
        .create_tenant(input)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::CREATED, Json(tenant)))
}

pub async fn get_tenant(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    if tenant.tenant_id != id && !tenant.is_admin {
        return Err(ApiError(relay_core::error::CoreError::Forbidden(
            "Access denied to tenant".to_string(),
        )));
    }

    let tenant_view = state
        .tenant_service
        .get_tenant(id)
        .await
        .map_err(ApiError)?
        .ok_or_else(|| {
            ApiError(relay_core::error::CoreError::NotFound(format!(
                "Tenant '{id}' not found"
            )))
        })?;

    Ok((StatusCode::OK, Json(tenant_view)))
}

pub async fn update_tenant(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateTenantInput>,
) -> Result<impl IntoResponse, ApiError> {
    if tenant.tenant_id != id && !tenant.is_admin {
        return Err(ApiError(relay_core::error::CoreError::Forbidden(
            "Access denied to update tenant".to_string(),
        )));
    }

    let tenant_view = state
        .tenant_service
        .update_tenant(id, input)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(tenant_view)))
}

pub async fn delete_tenant(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    if !tenant.is_admin {
        return Err(ApiError(relay_core::error::CoreError::Forbidden(
            "Admin privileges required to delete tenant".to_string(),
        )));
    }

    state
        .tenant_service
        .delete_tenant(id)
        .await
        .map_err(ApiError)?;

    Ok(StatusCode::NO_CONTENT)
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

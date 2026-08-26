use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use domain::dto::{CreateTransformationInput, TestTransformationInput, UpdateTransformationInput};

use crate::error::ApiError;
use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListTransformationsQuery {
    pub subscription_id: Option<Uuid>,
}

/// GET /transformations?subscription_id=... (Endpoint #58)
pub async fn list_transformations(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Query(query): Query<ListTransformationsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let subscription_id = query.subscription_id.ok_or_else(|| {
        ApiError(relay_core::error::CoreError::Validation(
            "Query parameter 'subscription_id' is required".to_string(),
        ))
    })?;

    let transformations = state
        .transformation_service
        .list_transformations(tenant.tenant_id, subscription_id)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(transformations)))
}

/// POST /transformations (Endpoint #59)
pub async fn create_transformation(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Json(input): Json<CreateTransformationInput>,
) -> Result<impl IntoResponse, ApiError> {
    let transformation = state
        .transformation_service
        .create_transformation(tenant.tenant_id, input)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::CREATED, Json(transformation)))
}

/// PATCH /transformations/{id} (Endpoint #60)
pub async fn update_transformation(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateTransformationInput>,
) -> Result<impl IntoResponse, ApiError> {
    let transformation = state
        .transformation_service
        .update_transformation(tenant.tenant_id, id, input)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(transformation)))
}

/// DELETE /transformations/{id} (Endpoint #61)
pub async fn delete_transformation(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .transformation_service
        .delete_transformation(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_transformation(
    State(state): State<AppState>,
    Json(input): Json<TestTransformationInput>,
) -> Result<impl IntoResponse, ApiError> {
    let transformed = state
        .transformation_service
        .test_transformation(&input.template, &input.payload)
        .await
        .map_err(ApiError)?;

    Ok((
        StatusCode::OK,
        Json(domain::dto::TestTransformationOutput {
            transformed_payload: transformed,
        }),
    ))
}

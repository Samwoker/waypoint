use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use domain::dto::{CreateSubscriptionInput, UpdateSubscriptionInput};

use crate::error::ApiError;
use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListPagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_subscriptions(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Query(pagination): Query<ListPagination>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = pagination.limit.unwrap_or(20);
    let offset = pagination.offset.unwrap_or(0);

    let subscriptions = state
        .subscription_service
        .list_subscriptions(tenant.tenant_id, limit, offset)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(subscriptions)))
}

pub async fn create_subscription(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Json(input): Json<CreateSubscriptionInput>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!(
        tenant_id = %tenant.tenant_id,
        source_id = %input.source_id,
        destination_id = %input.destination_id,
        "Creating new subscription"
    );

    let subscription = state
        .subscription_service
        .create_subscription(tenant.tenant_id, input)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::CREATED, Json(subscription)))
}

pub async fn get_subscription(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let subscription = state
        .subscription_service
        .get_subscription(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?
        .ok_or_else(|| ApiError(relay_core::error::CoreError::NotFound(format!("Subscription '{id}' not found"))))?;

    Ok((StatusCode::OK, Json(subscription)))
}

pub async fn update_subscription(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateSubscriptionInput>,
) -> Result<impl IntoResponse, ApiError> {
    let subscription = state
        .subscription_service
        .update_subscription(tenant.tenant_id, id, input)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(subscription)))
}

pub async fn delete_subscription(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .subscription_service
        .delete_subscription(tenant.tenant_id, id)
        .await
        .map_err(ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn pause_subscription(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let subscription = state
        .subscription_service
        .update_subscription(
            tenant.tenant_id,
            id,
            UpdateSubscriptionInput {
                event_types: None,
                filter_rules: None,
                transformation_template: None,
                is_active: Some(false),
            },
        )
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(subscription)))
}

pub async fn resume_subscription(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let subscription = state
        .subscription_service
        .update_subscription(
            tenant.tenant_id,
            id,
            UpdateSubscriptionInput {
                event_types: None,
                filter_rules: None,
                transformation_template: None,
                is_active: Some(true),
            },
        )
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(subscription)))
}

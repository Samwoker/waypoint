use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use domain::dto::LoginInput;

use crate::state::AppState;

pub async fn login(
    State(_state): State<AppState>,
    Json(_input): Json<LoginInput>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn refresh_token(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn me(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

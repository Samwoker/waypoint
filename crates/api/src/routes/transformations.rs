use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use domain::dto::TestTransformationInput;

use crate::state::AppState;

pub async fn test_transformation(
    State(_state): State<AppState>,
    Json(_input): Json<TestTransformationInput>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

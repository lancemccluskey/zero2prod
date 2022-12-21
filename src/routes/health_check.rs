use axum::{http, response::IntoResponse};

#[tracing::instrument(name = "Health check")]
pub async fn health_check() -> impl IntoResponse {
    http::StatusCode::OK
}

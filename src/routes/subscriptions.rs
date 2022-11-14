use axum::{extract::Form, http::StatusCode, response::IntoResponse};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(Form(_form): Form<FormData>) -> impl IntoResponse {
    StatusCode::OK
}

use std::sync::Arc;

use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    State(connection): State<Arc<PgPool>>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    let query_result = sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection.as_ref())
    .await;

    match query_result {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            println!("Failed to execute query: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

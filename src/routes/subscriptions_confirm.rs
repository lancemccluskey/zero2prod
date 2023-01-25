use anyhow::Context;
use axum::{
    extract::{Query, State},
    http,
    response::IntoResponse,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::startup::AppState;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum ConfirmError {
    #[error("{0}")]
    Unauthorized(String),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl std::fmt::Debug for ConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::util::error_chain_fmt(self, f)
    }
}

impl IntoResponse for ConfirmError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ConfirmError::Unauthorized(_) => http::StatusCode::UNAUTHORIZED.into_response(),
            ConfirmError::Unexpected(_) => http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(parameters, app_state),
    err(Debug)
)]
pub async fn confirm(
    State(app_state): State<AppState>,
    Query(parameters): Query<Parameters>,
) -> Result<impl IntoResponse, ConfirmError> {
    let id = get_subscriber_id_from_token(&app_state.pool, &parameters.subscription_token)
        .await
        .context("Failed to get subscriber id from token.")?;

    if let Some(subscriber_id) = id {
        confirm_subscriber(&app_state.pool, subscriber_id)
            .await
            .context("Failed to mark subscriber as confirmed")?;

        Ok(http::StatusCode::OK)
    } else {
        Err(ConfirmError::Unauthorized(
            "No associated subscriber id was found for the provided subscription token".into(),
        ))
    }
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed",
    skip(subscriber_id, pool),
    err(Debug)
)]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tracing::instrument(
    name = "Get subscriber_id from token",
    skip(subscription_token, pool),
    err(Debug)
)]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens \
      WHERE subscription_token = $1",
        subscription_token
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.subscriber_id))
}

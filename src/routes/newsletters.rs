use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Json, State},
    http,
    response::IntoResponse,
};
use sqlx::PgPool;

use crate::{domain::SubscriberEmail, startup::AppState};

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::util::error_chain_fmt(self, f)
    }
}

impl IntoResponse for PublishError {
    fn into_response(self) -> axum::response::Response {
        match self {
            PublishError::Unexpected(_) => http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

#[tracing::instrument(name = "Publish newsletter", skip(body, app_state))]
pub async fn publish_newsletter(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<BodyData>,
) -> Result<impl IntoResponse, PublishError> {
    let subscribers = get_confirmed_subscribers(&app_state.pool).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                app_state
                    .email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    .context(format!(
                        "Failed to send newsletter issue to {}",
                        subscriber.email
                    ))?;
                // ! Not working with `with_context` because `()` doesnt implement `Display`
                // .with_context(|| {
                //     format!("Failed to send newsletter issue to {}", subscriber.email);
                // })?;
            }
            Err(error) => {
                tracing::warn!(
                  // Record the error chain as a structured field
                  // on the log record
                  error.cause_chain = ?error,
                  "Skipping a confirmed subscriber. \
                  Their stored contact details are invalid."
                )
            }
        }
    }
    Ok(http::StatusCode::OK)
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
      SELECT email
      FROM subscriptions
      WHERE status = 'confirmed'
    "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => Err(anyhow::anyhow!(error)),
    })
    .collect();

    Ok(confirmed_subscribers)
}

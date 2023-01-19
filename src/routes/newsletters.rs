use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Json, State},
    http::{self, header},
    response::IntoResponse,
};
use secrecy::Secret;
use sqlx::PgPool;

use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    domain::SubscriberEmail,
    startup::AppState,
};

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
    #[error("Authentication failed.")]
    Auth(#[source] anyhow::Error),
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
            PublishError::Auth(_) => {
                let mut headers = header::HeaderMap::new();

                let header_value =
                    header::HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();

                headers.insert(header::WWW_AUTHENTICATE, header_value);

                (http::StatusCode::UNAUTHORIZED, headers).into_response()
            }
        }
    }
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(body, app_state, headers),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty),
    err(Debug)
)]
pub async fn publish_newsletter(
    headers: header::HeaderMap,
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<BodyData>,
) -> Result<impl IntoResponse, PublishError> {
    let credentials = basic_authentication(&headers).map_err(PublishError::Auth)?;
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    let user_id = validate_credentials(credentials, &app_state.pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => PublishError::Auth(e.into()),
            AuthError::Unexpected(_) => PublishError::Unexpected(e.into()),
        })?;
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

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
                // ! Not working with `with_context` because `()` doesn't implement `Display`
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

fn basic_authentication(headers: &header::HeaderMap) -> Result<Credentials, anyhow::Error> {
    // Header value must be a valid UTF8 string
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing.")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.")?;
    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;
    let decoded_bytes = base64::decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials.")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8.")?;

    // Split into two segments, using ':' as the delimiter
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth"))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
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

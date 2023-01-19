use std::sync::Arc;

use axum::{
    extract::{Form, State},
    http,
    response::IntoResponse,
};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, Secret};

use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    startup::AppState,
    util::error_chain_fmt,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    Auth(#[source] anyhow::Error),
    #[error("Something went wrong")]
    Unexpected(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for LoginError {
    fn into_response(self) -> axum::response::Response {
        match self {
            LoginError::Unexpected(_) => http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            // ! Not sure we even need this or should be impl IntoResponse
            LoginError::Auth(_) => http::StatusCode::UNAUTHORIZED.into_response(),
        }
    }
}

#[tracing::instrument(
  name = "Login",
  skip(form, app_state),
  fields(username=tracing::field::Empty, user_id=tracing::field::Empty),
  err(Debug)
)]
pub async fn login(
    State(app_state): State<Arc<AppState>>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &app_state.pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

            Ok((http::StatusCode::SEE_OTHER, [(http::header::LOCATION, "/")]))
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::Auth(e.into()),
                AuthError::Unexpected(_) => LoginError::Unexpected(e.into()),
            };

            let query_string = format!("error={}", urlencoding::Encoded::new(e.to_string()));

            let hmac_tag = {
                let mut mac = Hmac::<sha2::Sha256>::new_from_slice(
                    app_state.hmac_secret.0.expose_secret().as_bytes(),
                )
                .unwrap();
                mac.update(query_string.as_bytes());
                mac.finalize().into_bytes()
            };
            Err((
                http::StatusCode::SEE_OTHER,
                [(
                    http::header::LOCATION,
                    format!("/login?{query_string}&tag={hmac_tag:x}"),
                )],
            ))
        }
    }
}

use axum::response::{Html, IntoResponse};

#[tracing::instrument(name = "Home")]
pub async fn home() -> impl IntoResponse {
    // `Html` auto-sets `content-type` to `text/html`
    Html(include_str!("home.html"))
}

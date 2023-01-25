use axum::response::{Html, IntoResponse};
use axum_extra::extract::cookie::{Cookie, SignedCookieJar};

#[tracing::instrument(name = "Login form", skip(jar))]
pub async fn login_form(jar: SignedCookieJar) -> impl IntoResponse {
    let error_html = match jar.get("_flash") {
        None => "".into(),
        Some(cookie) => format!("<p><i>{}</i></p>", cookie.value()),
    };

    (
        jar.remove(Cookie::named("_flash")),
        Html(format!(
            r#"<!DOCTYPE html>
      <html lang="en">
        <head>
          <meta http-equiv="content-type" content="text/html; charset=utf-8" />
          <title>Login</title>
        </head>
        <body>
          {error_html}
          <form action="/login" method="post">
            <label
              >Username
              <input type="text" placeholder="Enter Username" name="username" />
            </label>
            <label
              >Password
              <input type="password" placeholder="Enter Password" name="password" />
            </label>
            <button type="submit">Login</button>
          </form>
        </body>
      </html>
      "#
        )),
    )
}

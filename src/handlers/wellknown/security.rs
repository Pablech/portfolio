use axum::{http::{header, StatusCode}, response::{IntoResponse, Response}};
use chrono::{Duration, Utc};

// route = "/.well-known/security.txt"
pub async fn security_txt() -> Response {
    let expires = (Utc::now() + Duration::days(90)).format("%Y-%m-%dT%H:%M:%S.000Z");
    let body = format!(
        "Contact: contato@pablech-dev.com\n\
        Expires: {}\n\
        Preferred-Languages: pt, en, es\n",
        expires
    );
    
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body,).into_response()
}
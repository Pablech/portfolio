use axum::{http::{header, StatusCode}, response::{IntoResponse, Response}};

// route = "/robots.txt"
pub async fn robots_txt_handler() -> Response {
    let body = "User-agent: *\nAllow: /\n";
    
    (StatusCode::OK,[(header::CONTENT_TYPE, "text/plain")], body,).into_response()
}
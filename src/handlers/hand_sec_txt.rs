use axum::{http::{header, StatusCode}, response::{IntoResponse, Response}};

// route = "/.well-known/security.txt"
pub async fn security_txt_handler() -> Response {
    let body = "Contact: contato@pablech-dev.com\n\
        Preferred-Languages: pt, en, es\n";
    
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/plain")], body,).into_response()
}
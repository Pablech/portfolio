use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

// route = "/favicon.ico"
pub async fn favicon_ico_handler() -> Response {
    let ico = include_bytes!("../../assets/favicon.ico");
    
    (StatusCode::OK, [(header::CONTENT_TYPE, "image/x-icon"),
         (header::CACHE_CONTROL, "public, max-age=86400")], ico,
    ).into_response()
}
use axum::{http::{header, StatusCode}, response::{IntoResponse, Response}};

// route = "/robots.txt"
pub async fn robots_txt_handler() -> Response {
    let body: &str = "User-agent: *\nAllow: /\n\nSitemap: https://pablech-dev.com/sitemap.xml";
    
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()
}
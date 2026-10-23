use axum::{http::header, Json};

use crate::handlers::models::BasicRes;

pub async fn keys_handler() -> ([(header::HeaderName, &'static str); 1], Json<BasicRes>) {
    let hotkeys = "Atalhos disponíveis:\n  \
        Alt + ↑ / ↓              - cicla entre terminais\n  \
        Alt + PageUp / PageDown  - altera entre primeiro/último terminal\n  \
        Alt + w                  - fecha o terminal com foco";
    
    ([(header::CACHE_CONTROL, "no-store")], Json(BasicRes { output: hotkeys.to_string() }))
}
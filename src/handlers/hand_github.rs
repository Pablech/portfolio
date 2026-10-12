use axum::{Json, http::header, extract::State};
use std::sync::Arc;

use crate::repositories::github::github::GitHub;
use crate::handlers::models::BasicRes;

// route = "/api/github"
pub async fn github_handler(State(gh): State<Arc<GitHub>>) -> ([(header::HeaderName, &'static str); 1], Json<BasicRes>) {
    let output = gh.get_github_json().await;
    ([(header::CACHE_CONTROL, "no-store")], Json(BasicRes { output: output }))
}


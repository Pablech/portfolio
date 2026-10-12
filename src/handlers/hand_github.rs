use axum::{Json, extract::State};
use std::sync::Arc;

use crate::repositories::github::github::GitHub;
use crate::handlers::models::BasicRes;

// route = "/api/github"
pub async fn github_handler(State(gh): State<Arc<GitHub>>) -> Json<BasicRes> {
    let output = gh.get_github_json().await;
    Json(BasicRes { output: output })
}

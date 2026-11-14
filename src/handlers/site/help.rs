use askama_axum::IntoResponse;
use axum::{extract::Path, http::StatusCode, response::Response};

const HELP_DEFAULT: &str = include_str!("../../../static/help/default.txt");
const HELP_SITE: &str    = include_str!("../../../static/help/site.txt");
const HELP_GAME: &str    = include_str!("../../../static/help/game.txt");

// router = GET /api/help/:kind  →  default + kind
pub async fn help(Path(kind): Path<String>) -> Response {
    let extra = match kind.as_str() {
        "os" => HELP_SITE,
        "games" => HELP_GAME,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    let text = format!("{}\n{}",extra.trim(), HELP_DEFAULT.trim_end());
    axum::Json(serde_json::json!({ "output": text })).into_response()
}
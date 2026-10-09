use std::sync::Arc;
use axum::{
    Router, extract::{FromRef, Request}, http::StatusCode, middleware::{self, Next}, response::Response, routing::{get, post}
};
use tower_http::services::ServeDir;
use tower_governor::{
    governor::GovernorConfigBuilder, 
    GovernorLayer,
    key_extractor::SmartIpKeyExtractor
};
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse};
use tracing::{warn, Level};

use crate::repositories::github::github::GitHub;
use crate::handlers::{home_handler, command_handler, send_email_handler, github_handler,};// debug_headers};
use crate::core::env_settings::EnvSetting;

#[derive(Clone)]
pub struct AppState {
    pub gh: Arc<GitHub>,
    pub env: Arc<EnvSetting>,
}

impl FromRef<AppState> for Arc<GitHub> {
    fn from_ref(state: &AppState) -> Arc<GitHub> {
        state.gh.clone()
    }
}

impl FromRef<AppState> for Arc<EnvSetting> {
    fn from_ref(state: &AppState) -> Arc<EnvSetting> {
        state.env.clone()
    }
}

pub async fn log_rate_limit(req: Request, next: Next) -> Response {

    let ip = req.headers().get("cf-connecting-ip")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default").to_string();
    let path = req.uri().path().to_string();

    let response = next.run(req).await;

    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        warn!("rate limit: IP {} on {}", ip, path);
    }
    response
}

pub fn create_router(gh: Arc<GitHub>, env: Arc<EnvSetting>) -> Router {
    let serve_assets = ServeDir::new("assets");
    let state = AppState { gh, env };

    let create_rate_limit = |per_sec: u64, burst: u32| {
        Box::leak(Box::new(
            GovernorConfigBuilder::default()
                .key_extractor(SmartIpKeyExtractor)
                .per_second(per_sec).burst_size(burst)
                .finish().unwrap()
            )
        )
    };

    let cmd_conf = create_rate_limit(1, 10);
    let gh_conf = create_rate_limit(1, 5);
    let email_conf = create_rate_limit(1, 2);

    Router::new()
        .route("/", get(home_handler))
        // .route("/debug/headers", get(debug_headers))
        .route("/api/cmd", post(command_handler)
                .layer(GovernorLayer {config: cmd_conf}))
        .route("/api/send-email", post(send_email_handler)
                .layer(GovernorLayer {config: email_conf}))
        .route("/api/github", post(github_handler)
            .layer(GovernorLayer {config: gh_conf}))
        .layer(middleware::from_fn(log_rate_limit))
        .layer(TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO)))
        .nest_service("/assets", serve_assets).with_state(state)
}
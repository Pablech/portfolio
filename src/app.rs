use std::{sync::Arc,};
use axum::{
    Router, 
    extract::FromRef, 
    middleware::self, 
    routing::{get, post}
};
use tower_http::services::ServeDir;
use tower_governor::{
    governor::GovernorConfigBuilder, 
    GovernorLayer,
    key_extractor::SmartIpKeyExtractor
};

use crate::repositories::github::github::GitHub;
use crate::handlers::{
    hand_help::help_handler,
    hand_github::github_handler,
    hand_email::send_email_handler,
    hand_home::home_handler,
    hand_about_me::about_handler,
    // hand_debug::debug_headers,
};
use crate::core::{
    env_settings::EnvSetting,
    log_conf::{log_rate_limit, request_logger},
    tartip::tarpit_scanners,
    ddos_protect::TarpitState,
};


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

pub fn create_router(gh: Arc<GitHub>, env: Arc<EnvSetting>, ) -> Router {
    let serve_assets = ServeDir::new("assets");
    let state = AppState { gh, env, };
    let tarpit = Arc::new(TarpitState::new());

    let create_rate_limit = |per_sec: u64, burst: u32| {
        Box::leak(Box::new(
            GovernorConfigBuilder::default()
                .key_extractor(SmartIpKeyExtractor)
                .per_second(per_sec).burst_size(burst)
                .finish().unwrap()
            )
        )
    };

    let gh_conf = create_rate_limit(1, 5);
    let email_conf = create_rate_limit(1, 2);

    Router::new()
        .route("/", get(home_handler))

        .route("/api/help", get(help_handler))

        .route("/api/about", get(about_handler))

        // .route("/debug/headers", get(debug_headers))
        .route("/api/send-email", post(send_email_handler)
                .layer(GovernorLayer {config: email_conf}))

        .route("/api/github", get(github_handler)
            .layer(GovernorLayer {config: gh_conf}))

            .layer(middleware::from_fn(request_logger))
            .layer(middleware::from_fn(log_rate_limit))
            .layer(middleware::from_fn_with_state(tarpit.clone(), tarpit_scanners))
        .nest_service("/assets", serve_assets).with_state(state)
}
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
use crate::handlers::{site, wellknown};
use crate::core::{
    env::EnvSetting,
    logging::{log_rate_limit, request_logger},
    tartip::tarpit_scanners,
    ddos::TarpitState,
    security_header::security_header,
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
    let state = AppState { gh, env };
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

    let api_router = Router::new()
        .route("/help", get(site::help))
        .route("/about", get(site::about))
        .route("/keys", get(site::keys))
        .route("/resume", get(site::resume))
        .route("/send-email", post(site::send_email).layer(GovernorLayer { config: email_conf }))
        .route("/github", get(site::github).layer(GovernorLayer { config: gh_conf }));

    let bot_router = Router::new()
        .route("/.well-known/security.txt", get(wellknown::security_txt))
        .route("/robots.txt", get(wellknown::robots_txt))
        .route("/llms.txt", get(wellknown::llms_txt))
        .route("/favicon.ico", get(wellknown::favicon))
        .route("/sitemap.xml", get(wellknown::sitemap));

    Router::new()
        .route("/", get(site::home))
        .merge(bot_router)
        .nest("/api", api_router)
        // .nest("/game/ws", game_conn_4_router)
        .layer(middleware::from_fn(security_header))
        .layer(middleware::from_fn(request_logger))
        .layer(middleware::from_fn(log_rate_limit))
        .layer(middleware::from_fn_with_state(tarpit.clone(), tarpit_scanners))
        .nest_service("/assets", serve_assets)
        .with_state(state)
}
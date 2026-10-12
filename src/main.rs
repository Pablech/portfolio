use tracing::{Level, warn};
use tokio::net::TcpListener;
use std::sync::Arc;
use std::net::SocketAddr;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, filter::Targets};

mod core;
mod repositories;
mod app;
mod handlers;

use repositories::github::github::GitHub;
use repositories::httpc::httpc::HttpC;
use core::env_settings::EnvSetting;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let filter = Targets::new()
        .with_target("site", Level::INFO)
        .with_target("tower_http", Level::INFO)
        .with_target("reqwest", Level::INFO);
    
    let file_appender = tracing_appender::rolling::daily("logs", "portfolio.logs");
    let (filter_writer, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(filter).with(
            fmt::layer().with_target(false)
            .with_file(false).with_line_number(false).compact())
        .with(fmt::layer().with_ansi(false).with_writer(filter_writer)).init();

    let env = EnvSetting::new();
    let httpc = HttpC::new(&env);

    if !httpc.ready() {warn!("HttpC is not ready"); return Err("HttpC not ready".into());}

    let gh = GitHub::new(&env, &httpc);
    let shared_gh = Arc::new(gh);
    let shared_env = Arc::new(env);

    let app = app::create_router(shared_gh, shared_env, );
    let addr  = "localhost:3000";
    tracing::debug!("Listening on: {}", addr);
    let listener = TcpListener::bind(addr).await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>()
    )
    .await.unwrap();

    Ok(())
}

use tracing::warn;
use tokio::net::TcpListener;
use std::sync::Arc;
use std::net::SocketAddr;

use site::{
    core::{
        logging::configure_log,
        env::EnvSetting,
    },
    repositories::{
        github::github::GitHub,
        httpc::httpc::HttpC,
        games::GameHub,
    },
    app::create_router,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let _log_guard = configure_log();

    let env = EnvSetting::new();
    let httpc = HttpC::new(&env);
    let addr  = format!("{}:{}", &env.host, &env.port);

    if !httpc.ready() {warn!("HttpC is not ready"); return Err("HttpC not ready".into());}

    let gh = GitHub::new(&env, &httpc);
    let games = GameHub::new(env.reap_grace, env.orphan_ttl);

    let shared_gh = Arc::new(gh);
    let shared_env = Arc::new(env);
    let shared_games = Arc::new(games);


    let app = create_router(shared_gh, shared_env, shared_games);
    tracing::info!("Listening on: {}", addr);
    let listener = TcpListener::bind(addr).await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>()
    )
    .await.unwrap();

    Ok(())
}

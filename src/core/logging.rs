use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::StatusCode
};
use tracing::{warn, info, Level};
use time::{
    macros::format_description,
    UtcOffset,
};
use tracing_subscriber::{
    fmt, 
    layer::SubscriberExt, 
    util::SubscriberInitExt, 
    filter::Targets,
    fmt::time::OffsetTime,
};

use crate::core::extract::{
    user_agent::get_ua,
    ip::get_ip,
};

pub async fn log_rate_limit(req: Request, next: Next) -> Response {

    let ip = get_ip(&req);
    let path = req.uri().path().to_string();
    let ua = get_ua(&req);

    let response = next.run(req).await;

    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        warn!("rate limit: IP {} on {} - user-agent: {}", ip, path, ua);
    }
    response
}

pub async fn request_logger(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let ua = get_ua(&req);

    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let latency = start.elapsed();
    let status = response.status();

    info!(
        method = %method,
        status = %status,
        latency_ms = latency.as_millis() as u64,
        uri = %uri,
        ua = &ua,
        "http"
    );

    response
}

pub fn configure_log() -> tracing_appender::non_blocking::WorkerGuard {
    let timer = OffsetTime::new(
        UtcOffset::from_hms(-3, 0, 0).unwrap(),
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    );

    let filter = Targets::new()
        .with_target("site", Level::INFO)
        .with_target("tower_http", Level::INFO)
        .with_target("reqwest", Level::INFO);
    
    let file_appender = tracing_appender::rolling::daily("logs", "portfolio.logs");
    let (filter_writer, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(filter).with(
            fmt::layer().with_timer(timer.clone()).with_target(false)
            .with_file(false).with_line_number(false).compact())
        .with(fmt::layer().with_timer(timer).with_target(false).with_ansi(false).with_writer(filter_writer)).init();
    guard
}
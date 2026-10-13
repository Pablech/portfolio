use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::StatusCode
};
use tracing::{warn, info};

use crate::core::get_ip::get_ip;
use crate::core::get_user_agent::get_ua;

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
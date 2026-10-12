use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::StatusCode
};
use tracing::{warn, info};

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

pub async fn request_logger(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    // let ip = req.headers().get("cf-connecting-ip")
    //     .and_then(|v| v.to_str().ok())
    //     .unwrap_or("default").to_string();
    
    // let ua: String = req.headers()
    //     .get("user-agent")
    //     .and_then(|v| v.to_str().ok())
    //     .unwrap_or("-")
    //     .chars()
    //     .take(120)
    //     .collect();

    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let latency = start.elapsed();
    let status = response.status();

    info!(
        method = %method,
        status = %status,
        latency_ms = latency.as_millis() as u64,
        // ip = &ip,
        uri = %uri,
        // user_agent = %ua,
        "http"
    );

    response
}
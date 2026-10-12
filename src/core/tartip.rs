use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use rand::Rng;

use crate::core::ddos_protect::TarpitState;

pub async fn tarpit_scanners(State(tarpit): State<Arc<TarpitState>>,
        req: Request<axum::body::Body>, next: Next, ) -> Response {

    let path = req.uri().path();

    let is_probe = path.ends_with(".php")
        || path.contains("phpinfo")
        || path.contains("wp-")
        || path.contains(".env")
        || path.contains(".git")
        || path.contains("admin")
        || path.contains(".ico");

    if is_probe {
        let ip = req.headers()
            .get("cf-connecting-ip")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        if tarpit.is_blocked(&ip) {
            tracing::warn!("TARPIT: {} já visitou recentemente, {}", ip,
                StatusCode::SERVICE_UNAVAILABLE,
            );
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }

        tarpit.block(&ip);

        let delay_seconds = rand::thread_rng().gen_range(25..=50);
        tracing::warn!(
            "TARPIT: Scanner {} capturado em {} - delay de {}s",
            ip, path, delay_seconds
        );

        sleep(Duration::from_secs(delay_seconds)).await;

        let fake_phpinfo = r#"<!DOCTYPE html>
            <html><body>
            <table border='1'><tr><th>PHP Version</th><td>8.2.14</td></tr>
            <tr><th>System</th><td>Linux portfolio 5.15.0-generic-fuck-u-puto</td></tr>
            </table>
            </body></html>"#;

        return (StatusCode::OK, [("content-type", "text/html")],
            fake_phpinfo, ).into_response();
    }

    next.run(req).await
}
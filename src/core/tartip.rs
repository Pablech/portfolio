use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use rand::Rng;

use crate::core::{
    ddos_protect::TarpitState,
    get_ip::get_ip,
    get_user_agent::get_ua,
};

pub async fn tarpit_scanners(State(tarpit): State<Arc<TarpitState>>,
        req: Request<axum::body::Body>, next: Next, ) -> Response {

    let path = req.uri().path();

    let is_probe = path.contains(".php")
        || path.contains("phpinfo")
        || path.contains("wp-")
        || path.contains(".env")
        || path.contains(".git")
        || path.contains("config")
        || path.contains(".md");

    if is_probe {
        let ip = get_ip(&req);
        let ua = get_ua(&req);
        
        if ip != "unknown" {
            if tarpit.is_blocked(&ip) {
                let sc_u = StatusCode::SERVICE_UNAVAILABLE;
                tracing::warn!("TARPIT: Scanner ip={} status={} path={} user-agent={}", ip, sc_u, path, ua,);
                return sc_u.into_response();
            }

            tarpit.block(&ip);
        }

        let delay_seconds = rand::thread_rng().gen_range(80..=120);
        let sc_ok = StatusCode::OK;
        tracing::warn!(
            "TARPIT: Scanner ip={} status={} delay={} path={} user-agent={}",
            ip, sc_ok, delay_seconds, path, ua,
        );

        sleep(Duration::from_secs(delay_seconds)).await;

        let fake_phpinfo = r#"<!DOCTYPE html>
            <html><body>
            <table border='1'><tr><th>PHP Version</th><td>8.2.14</td></tr>
            <tr><th>System</th><td>Linux portfolio 5.15.0-generic-fuck-u-puto</td></tr>
            </table>
            </body></html>"#;

        return (sc_ok, [("content-type", "text/html")],
            fake_phpinfo, ).into_response();
    }

    next.run(req).await
}
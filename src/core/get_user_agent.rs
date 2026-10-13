use axum::extract::Request;

pub fn get_ua(req: &Request<axum::body::Body>) -> String {
    req.headers().get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown").chars()
        .take(120).collect()
}
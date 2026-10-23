use axum::http::Request;

pub fn get_ip(req: &Request<axum::body::Body>, ) -> String {
    req.headers()
        .get("cf-connecting-ip")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            req.headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.split(',').next().unwrap_or(s).trim())
        })
        .unwrap_or("unknown")
        .to_string()
}

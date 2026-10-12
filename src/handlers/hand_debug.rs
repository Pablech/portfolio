use axum::extract::Request;

// route = "/api/debug"
pub async fn debug_headers(req: Request) -> String {
    let headers: Vec<_> = req
        .headers()
        .iter()
        .filter(|(k, _)| k.as_str().contains("for") || k.as_str().contains("connecting"))
        .map(|(k, v)| format!("{}: {}", k.as_str(), v.to_str().unwrap_or("invalid")))
        .collect();
    headers.join("\n")
}
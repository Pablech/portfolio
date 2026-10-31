use axum::{
    http::header,
    response::IntoResponse,
};

const RESUME_PDF_BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/resume.pdf"));

// route = "/api/resume"
pub async fn resume_pdf_handler() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (header::CONTENT_DISPOSITION, "inline; filename=\"curriculo-pablo.pdf\""),
            (header::CACHE_CONTROL, "no-store"),
        ],
        RESUME_PDF_BYTES,
    )
}
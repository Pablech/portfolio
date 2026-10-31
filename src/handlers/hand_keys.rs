use std::sync::LazyLock;
use axum::{http::header, Json};

use crate::handlers::{
    models::{BasicRes, SectionContent},
    formatters::format_section,
};

const KEYS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/keys.txt"));

static KEYS_OUTPUT: LazyLock<String> = LazyLock::new(|| {
    format_section(&[
        SectionContent {
            title: "Atalhos disponíveis", content: KEYS
        }
    ])
});

pub async fn keys_handler() -> ([(header::HeaderName, &'static str); 1], Json<BasicRes>) {
    ([(header::CACHE_CONTROL, "no-store")], Json(BasicRes { output: KEYS_OUTPUT.clone() }))
}
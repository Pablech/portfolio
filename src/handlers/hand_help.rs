use std::sync::LazyLock;
use axum::{http::header, Json};

use crate::handlers::{
    models::{BasicRes, SectionContent},
    formatters::format_section,
};

const HELP: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/help.txt"));

static HELP_OUTPUT: LazyLock<String> = LazyLock::new(|| {
    format_section(&[
        SectionContent {
            title: "Comandos disponíveis", content: HELP
        }
    ])
});

pub async fn help_handler() -> ([(header::HeaderName, &'static str); 1], Json<BasicRes>) {
    ([(header::CACHE_CONTROL, "no-store")], Json(BasicRes { output: HELP_OUTPUT.clone() }))
}
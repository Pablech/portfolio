use axum::{http::header, Json};
use include_dir::{include_dir, Dir};
use std::sync::LazyLock;

use crate::handlers::{
    models::{BasicRes, SectionContent, Project},
    formatters::format_section,
};

const ABOUT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/about.txt"));
const PHILOSOPHY: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/philosophy.txt"));
const CONTACT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/contact.txt"));

static PROJECTS_DIR: Dir = include_dir!("static/projects");

static DYNAMIC_PROJECTS: LazyLock<Vec<Project>> = LazyLock::new(|| {
    let mut projects = Vec::new();
    
    for entry in PROJECTS_DIR.dirs() {
        let folder_name = entry.path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("projeto");
        
        let name  = read_field(&entry, "name.txt").unwrap_or(folder_name);
        let stack = read_field(&entry, "stack.txt").unwrap_or("");
        let desc  = read_field(&entry, "desc.txt").unwrap_or("");
        
        projects.push(Project { name, stack, desc });
    }

    projects
});

static ABOUT_OUTPUT: LazyLock<String> = LazyLock::new(|| {
    let projects_body = format_section(&*DYNAMIC_PROJECTS);

    let sections = &[
        SectionContent { title: "Sobre mim", content: ABOUT },
        SectionContent { title: "Projetos", content: &projects_body },
        SectionContent { title: "Filosofia", content: PHILOSOPHY },
        SectionContent { title: "Contato", content: CONTACT },
    ];

    format_section(sections)
});

fn read_field(entry: &Dir<'static>, file_name: &str) -> Option<&'static str> {
    entry.files()
        .find(|f| f.path().file_name().and_then(|n| n.to_str()) == Some(file_name))
        .and_then(|f| f.contents_utf8())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

// route = "/api/about"
pub async fn about_handler() -> ([(header::HeaderName, &'static str); 1], Json<BasicRes>) {
    ([(header::CACHE_CONTROL, "no-store")], Json(BasicRes { output: ABOUT_OUTPUT.clone() }))
}
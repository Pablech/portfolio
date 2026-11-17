use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Repo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub private: bool,
    pub description: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RepoContent {
    pub name: String,
    pub path: String,
    pub r#type: String,
    pub size: u32,

    #[serde(rename = "_links")]
    pub links: RepoLinks,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RepoLinks {
    pub git: String,
    #[serde(rename = "self")]
    pub self_link: Option<String>,
    pub html: String,
}

#[derive(Serialize)]
pub struct RepoFile {
    pub name: String,
    pub code: String,
}

#[derive(Serialize)]
pub struct RepoInfo {
    pub id: u64,
    pub name: String,
    pub desc: String,
    pub lang: String,
    pub url: String,
    pub files: Vec<RepoFile>,
}
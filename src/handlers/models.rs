use serde::{Deserialize, Serialize};
use askama::Template;


#[derive(Serialize)]
pub struct BasicRes {
    pub output: String,
}

#[derive(Deserialize)]
pub struct SenfEmailReq {
    pub from: String,
    pub subject: String,
    pub body: String,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct PortfolioTemplate {
    pub title: String,
}

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
    pub timestamp: i64,
}

pub struct Project {
    pub name: &'static str,
    pub desc: &'static str,
    pub stack: &'static str,
}

pub trait TerminalRenderable {
    fn header(&self) -> String;
    fn sections(&self) -> Vec<(String, String)>;
}

pub struct SectionContent<'a> {
    pub title: &'a str,
    pub content: &'a str,
}

impl TerminalRenderable for Project {
    fn header(&self) -> String {
        self.name.to_string()
    }

    fn sections(&self) -> Vec<(String, String)> {
        let mut blocks = Vec::new();

        if !self.desc.trim().is_empty() {
            blocks.push(("Descrição".to_string(), self.desc.to_string()));
        }
        if !self.stack.trim().is_empty() {
            blocks.push(("Stacks".to_string(), self.stack.to_string()));
        }

        blocks
    }
}

impl<'a> TerminalRenderable for SectionContent<'a> {
    fn header(&self) -> String {
        self.title.to_string()
    }

    fn sections(&self) -> Vec<(String, String)> {
        if self.title == "Projetos" {
            vec![("@RAW".to_string(), self.content.to_string())]
        } else {
            vec![("".to_string(), self.content.to_string().trim().to_string())]
        }
    }
}
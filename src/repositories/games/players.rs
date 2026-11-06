use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Cpu {
    pub name: String,
    pub fig: String,
    pub player_fig: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Human {
    pub name: String,
    pub fig: String,
}

#[derive(Debug, Clone)]
pub enum AgentKind {
    Human,
    Cpu,
}

pub struct Agent {
    pub kind: AgentKind,
    pub name: String,
    pub fig: String,
    pub token: Option<String>,
}

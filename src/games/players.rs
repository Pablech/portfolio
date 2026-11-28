use crate::games::{Agent, AgentKind, Human, utils::{choice_fig, resolve_name}};

impl Human {
    pub fn new(name: String, fig: char) -> Self {
        Self { name, fig }
    }
}

impl Agent {
    pub fn new(kind: AgentKind, name: Option<String>, fig: Option<char>, seat: usize, 
            taken_figs: &[char], taken_names: &[String]) -> Self {
        
        let fig = fig.unwrap_or_else(|| choice_fig(taken_figs));
        let default_name = format!("Player {}", seat + 1);
        let name = resolve_name(name, &default_name, seat, taken_names);
        
        Self { kind, name, fig, token: None, default_name, default_fig: fig }
    }
}
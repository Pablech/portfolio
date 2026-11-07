use anyhow::Error;

use crate::repositories::games::{Agent, CpuBrain, GameHub, connect4::{Conn4Table, SmartishBrain}, generate_token, structs::AgentKind};

pub enum GameKind {
    Connect4,
}

pub fn build_agents(p1_name: String, p1_fig: char, vs_cpu: bool) -> Vec<Agent> {
    let p1 = Agent {
        kind: AgentKind::Human,
        name: p1_name,
        fig: p1_fig,
        token: Some(generate_token()),
    };
    let p2 = if vs_cpu {
        Agent { kind: AgentKind::Cpu, name: "Cpu.ai".into(), fig: '@'.into(), token: None }
    } else {
        Agent { kind: AgentKind::Human, name: "p2".into(), fig: '&'.into(), token: Some(generate_token()) }
    };
    vec![p1, p2]
}

impl GameKind {
    pub fn create(&self, hub: &GameHub, p1_name: String, p1_fig: char, vs_cpu: bool)
        -> Result<(String, String), Error>
    {
        let agents = build_agents(p1_name, p1_fig, vs_cpu);
        match self {
            Self::Connect4 => {
                let brains: Vec<Option<Box<dyn CpuBrain<Conn4Table>>>> = if vs_cpu {
                    vec![None, Some(Box::new(SmartishBrain))]
                } else {
                    vec![None, None]
                };
                hub.create_room(agents, brains)
            }
        }
    }
}

use anyhow::{Error, anyhow};

use crate::repositories::games::{
    Agent, AgentKind, CpuBrain, CreatedRoom, GameHub, GameKind, connect4::{Conn4Table, SmartishBrain}, generate_token, 
};

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
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "connect4" => Ok(Self::Connect4),
            other => Err(anyhow!("jogo desconhecido: {}", other)),
        }
    }

    pub fn create(&self, hub: &GameHub, p1_name: String, p1_fig: char, vs_cpu: bool)
        -> Result<CreatedRoom, Error>
    {
        let agents = build_agents(p1_name, p1_fig, vs_cpu);

        let host_token = agents[0].token.clone()
            .ok_or_else(|| anyhow!("host sem token"))?;
        let guest_token = if vs_cpu { None } else {
            agents[1].token.clone()
        };

        match self {
            Self::Connect4 => {
                let brains: Vec<Option<Box<dyn CpuBrain<Conn4Table>>>> = if vs_cpu {
                    vec![None, Some(Box::new(SmartishBrain))]
                } else {
                    vec![None, None]
                };
                let (room_id, _) = hub.create_room(agents, brains)?;
                Ok(CreatedRoom { room_id, host_token, guest_token })
            }
        }
    }
}

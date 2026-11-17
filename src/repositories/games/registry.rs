use std::sync::Arc;

use crate::repositories::games::{
    Agent, AgentKind, CpuBrain, CreatedRoom, GameError, GameHub, GameKind, GameLogic, connect4::{Conn4Table, SmartishBrain}, generate_token, 
};

pub fn build_agents(host_name: Option<String>, host_fig: Option<char>, vs_cpu: bool, seats: usize) -> Vec<Agent> {
    let mut taken_figs: Vec<char> = Vec::with_capacity(seats);
    let mut taken_names: Vec<String> = Vec::with_capacity(seats);
    let mut agents: Vec<Agent> = Vec::with_capacity(seats);

    for seat in 0..seats {
        let (kind, name, fig) = match seat {
            0 => (AgentKind::Human, host_name.clone(), host_fig),
            _ if vs_cpu => (AgentKind::Cpu, None, None),
            _ => (AgentKind::Human, None, None),
        };
        let agent = Agent::new(kind, name, fig, seat, &taken_figs, &taken_names);
        taken_figs.push(agent.fig);
        taken_names.push(agent.name.clone());
        agents.push(agent);
    }

    agents[0].token = Some(generate_token());
    agents
}

impl GameKind {
    pub fn from_str(s: &str) -> Result<Self, GameError> {
        match s {
            "connect4" => Ok(Self::Connect4),
            other => Err(GameError::ErrorProtocol(format!("jogo desconhecido: {}", other))),
        }
    }

    pub fn create(&self, hub: &Arc<GameHub>, host_name: Option<String>, host_fig: Option<char>, vs_cpu: bool)
        -> Result<CreatedRoom, GameError> {

        match self {
            Self::Connect4 => {
                let seats = Conn4Table::SEATS;
                let agents = build_agents(host_name, host_fig, vs_cpu, seats);

                let brains: Vec<Option<Box<dyn CpuBrain<Conn4Table>>>> = if vs_cpu {
                    (0..seats).map(|i| match i {
                        0 => None,
                        _ => Some(Box::new(SmartishBrain) as Box<dyn CpuBrain<Conn4Table>>),
                    }).collect()
                } else {
                    vec![None, None]
                };
                let (room_id, host_token) = hub.create_room(agents, brains)?;
                Ok(CreatedRoom { room_id, host_token })
            }
        }
    }
}

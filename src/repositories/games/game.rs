use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::Mutex;
use anyhow::{Error, anyhow};

use crate::repositories::games::{
    Agent, CpuBrain, GameError, GameHub, GameLogic, GameRoom, GameState, 
    RoomHandle, generate_token, ws::MoveOutcome, 
};

impl GameHub {
    pub fn new() -> Self {
        Self { rooms: DashMap::new() }
    }

    pub fn create_room<L: GameLogic>(&self, agents: Vec<Agent>, brains: Vec<Option<Box<dyn CpuBrain<L>>>>,) 
        -> Result<(String, String), Error> {

            let room_id = generate_token()[..8].to_string();
            let token = agents.first()
                .and_then(|a| a.token.clone())
                .ok_or_else(|| anyhow!("sala sem agente host com token"))?;

            let room = GameRoom::<L> {
                room_id: room_id.clone(),
                game: L::new(),
                agents,
                turn: 0,
                finished: false,
                brains,
            };

            self.rooms.insert(
                room_id.clone(),
                Arc::new(Mutex::new(room)) as Arc<dyn RoomHandle>,
            );

            Ok((room_id, token))
        }
}

impl<L: GameLogic> GameRoom<L> {
    fn resolve_after_place(&mut self) -> Result<MoveOutcome, GameError> {
        loop {
            let fig = self.turn as u8 + 1;

            if self.game.check_win(fig) {
                self.finished = true;
                return Ok(MoveOutcome::Win { 
                    output: self.game.output(&self.agents), 
                    winner: self.agents[self.turn].name.clone(), 
                });
            }

            if self.game.check_draw() {
                self.finished = true;
                return Ok(MoveOutcome::Draw { 
                    output: self.game.output(&self.agents) 
                });
            }

            if self.brains[(self.turn + 1) % self.agents.len()].is_none() {
                self.turn = (self.turn + 1) % self.agents.len();
                break;
            }

            self.turn = (self.turn + 1) % self.agents.len();

            if let Some(brain) = self.brains[self.turn].take() {
                let fig = self.turn as u8 + 1;
                let legal = self.game.legal_moves();
                let mv = brain.choose(&self.game, &legal, fig);
                self.game.place(mv, fig)?;
                self.brains[self.turn] = Some(brain);
            }
        }

        Ok(MoveOutcome::Continue { 
            output: self.game.output(&self.agents), 
            next_player: self.agents[self.turn].name.clone(),
        })
    }
}

#[async_trait::async_trait]
impl <L: GameLogic> RoomHandle for Mutex<GameRoom<L>> {
    async fn join(&self, token: &str) -> Result<GameState, GameError> {
        let mut room = self.lock().await;

        for a in &mut room.agents {
            if a.token.as_deref() == Some(token) {
                return Ok(GameState::JoinInfo { 
                    joined: a.name.clone(), 
                    room_id: room.room_id.clone()
                });
            }
        }
        Err(GameError::ErrorJoin("token inválido para esta sala".into()))
    }

    async fn play(&self, token: &str, input: &str) -> Result<GameState, GameError> {
        let mut room = self.lock().await;
        if room.finished {
            return Err(GameError::ErrorMove("partida encerrada".into()));
        }

        let agent_indx = room.agents.iter().position(|a| a.token.as_deref() == Some(token))
            .ok_or_else(|| GameError::ErrorJoin("token inválido".into()))?;

        if agent_indx != room.turn {
            return Err(GameError::ErrorMove("não é seu turno".into()));
        }

        let legal = room.game.legal_moves();
        let mv: usize = input.trim().parse()
            .map_err(|_| GameError::ErrorMove(format!("entrada {} inválida", input)))?;

        if !legal.contains(&mv) {
            return Err(GameError::ErrorMove(format!("movimento {} ilegal", mv)));
        }

        let player = room.turn as u8 + 1;
        room.game.place(mv, player)?;
        let outcome = room.resolve_after_place()?;
        Ok(GameState::State(outcome))
    }
}

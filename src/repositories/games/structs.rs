use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::repositories::games::ws::{GameError, GameState};

pub trait GameLogic: Send + Sync + 'static {
    fn new() -> Self where Self: Sized;
    fn legal_moves(&self) -> Vec<usize>;
    fn output(&self, players: &[Agent]) -> String;
    fn place(&mut self, mv: usize, player: u8) -> Result<(), GameError>;
    fn check_win(&self, fig: u8) -> bool;
    fn check_draw(&self) -> bool;
}

pub struct GameRoom<L: GameLogic> {
    pub room_id: String,
    pub game: L,
    pub agents: Vec<Agent>,
    pub turn: usize,
    pub finished: bool,
    pub brains: Vec<Option<Box<dyn CpuBrain<L>>>>,
}

pub trait CpuBrain<L: GameLogic>: Send + Sync + 'static {
    fn choose(&self, table: &L, legal: &[usize], fig: u8) -> usize;
}

#[async_trait::async_trait]
pub trait RoomHandle: Send + Sync {
    async fn join(&self, token: &str) -> Result<GameState, GameError>;
    async fn play(&self, token: &str, input: &str) -> Result<GameState, GameError>;
}

pub struct GameHub {
    pub rooms: DashMap<String, Arc<dyn RoomHandle>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Cpu {
    pub name: String,
    pub fig: char,
    pub players_fig: Vec<char>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Human {
    pub name: String,
    pub fig: char,
}

#[derive(Debug, Clone)]
pub enum AgentKind {
    Human,
    Cpu,
}

pub struct Agent {
    pub kind: AgentKind,
    pub name: String,
    pub fig: char,
    pub token: Option<String>,
}

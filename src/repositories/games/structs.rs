use std::sync::Arc;
use dashmap::DashMap;

use crate::repositories::games::{Agent, ws::{GameError, GameState}};

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
    fn choose(&self, table: &L, legal: &[usize], me: u8) -> usize;
}

#[async_trait::async_trait]
pub trait RoomHandle: Send + Sync {
    async fn join(&self, token: &String) -> Result<GameState, GameError>;
    async fn play(&self, token: &String, input: &str) -> Result<GameState, GameError>;
}

pub struct GameHub {
    pub rooms: DashMap<String, Arc<dyn RoomHandle>>,
}

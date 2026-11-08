use crate::repositories::games::domain::{
    enums::{GameError, GameState, PlayResult}, 
    structs::Agent
};

pub trait GameLogic: Send + Sync + 'static {
    fn new() -> Self where Self: Sized;
    fn legal_moves(&self) -> Vec<usize>;
    fn output(&self, players: &[Agent]) -> String;
    fn place(&mut self, mv: usize, player: u8) -> Result<(), GameError>;
    fn check_win(&self, fig: u8) -> bool;
    fn check_draw(&self) -> bool;
}

pub trait CpuBrain<L: GameLogic>: Send + Sync + 'static {
    fn choose(&self, table: &L, legal: &[usize], fig: u8) -> usize;
}

#[async_trait::async_trait]
pub trait RoomHandle: Send + Sync {
    async fn join(&self, token: &str, req_id: Option<&str>) -> Result<GameState, GameError>;
    async fn play(&self, token: &str, input: &str, req_id: Option<&str>) -> Result<PlayResult, GameError>;
    async fn rematch(&self, token: &str, accept: bool, req_id: Option<&str>) -> Result<PlayResult, GameError>;
}
use tokio::sync::Mutex;

use crate::{handlers::structs::ClaimedSeat, repositories::games::{
    GameState, PlayResult, 
    domain::{enums::GameError, structs::GameRoom, 
    traits::{GameLogic, RoomHandle}},
}};

#[async_trait::async_trait]
impl <L: GameLogic> RoomHandle for Mutex<GameRoom<L>> {
    async fn join(&self, token: &str, req_id: Option<&str>) -> Result<GameState, GameError> {
        let mut room = self.lock().await;

        room.seat_info(token, req_id)
    }

    async fn play(&self, token: &str, input: &str, req_id: Option<&str>) -> Result<PlayResult, GameError> {
        let mut room = self.lock().await;

        room.aply_moves(token, input, req_id)
    }

    async fn rematch(&self, token: &str, accept: bool, req_id: Option<&str>) -> Result<PlayResult, GameError> {
        let mut room = self.lock().await;

        room.vote_rematch(token, accept, req_id)
    }

    async fn claim(&self, name: Option<String>, fig: Option<char>) -> Result<ClaimedSeat, GameError> {
        let mut room = self.lock().await;

        room.claim_seat(name, fig)
    }

    async fn snapshot(&self) -> GameState {
        let room = self.lock().await;
        
        room.snapshot_state()
    }

    async fn rematch_cycle(&self) -> u64 {
        let room = self.lock().await;
        
        room.rematch_round()
    }

    async fn timeout_rematch(&self, cycle: u64) -> Option<GameState> {
        let room = self.lock().await;

        room.rematch_timeout(cycle)
    }

    async fn chat(&self, token: &str, text: &str) -> Result<String, GameError> {
        let mut room = self.lock().await;

        room.chat_line(token, text)
    }

    async fn leave(&self, token: &str) -> Result<String, GameError> {
        let mut room = self.lock().await;

        room.mark_left(token)
    }

    async fn quit(&self, token: &str) -> Result<String, GameError> {
        let mut room = self.lock().await;
        
        room.vacate_seat(token)
    }
}
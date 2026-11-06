use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum MoveOutcome {
    Continue { output: String, next_player: String },
    Win { output: String, winner: String },
    Draw { output: String },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum GameError {
    ErrorMove(String),
    ErrorJoin(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum GameState {
    JoinInfo { joined: String, room_id: String },
    State(MoveOutcome),
    Error(GameError),
}

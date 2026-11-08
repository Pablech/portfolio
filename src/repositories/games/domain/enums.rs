use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MoveOutcome {
    Continue { output: String, next_player: String },
    Win { output: String, winner: String },
    Draw { output: String },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum GameError {
    ErrorMove(String),
    ErrorJoin(String),
    ErrorValidation(String),
}

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum GameState {
    JoinInfo { joined: String, room_id: String },
    State(MoveOutcome),
    Error(GameError),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", content = "data", rename_all = "lowercase")]
pub enum ClientMsg {
    Join { token: String, req_id: Option<String> },
    Play { token: String, input: String, req_id: Option<String> }
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ServerMsg {
    Joined { name: String, room_id: String },
    Error { message: String, fatal: bool },
    State { state: GameState, req_id: Option<String> },
}

#[derive(Debug, Clone)]
pub enum AgentKind {
    Human,
    Cpu,
}

#[derive(Debug)]
pub enum PlayResult {
    Fresh(GameState),
    Replayed(GameState),
}

pub enum GameKind {
    Connect4,
}
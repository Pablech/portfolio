use std::sync::Arc;
use axum::extract::ws::Message;
use serde::Deserialize;
use tokio::sync::{broadcast, mpsc};

use crate::repositories::games::{GameHub, GameState, LiveRoom, RoomHandle};

#[derive(Deserialize)]
pub struct WsParams {
    pub token: Option<String>,
}

pub struct Session {
    pub hub: Arc<GameHub>,
    pub live: Arc<LiveRoom>,
    pub room: Arc<dyn RoomHandle>,
    pub room_id: String,
    pub tx: broadcast::Sender<GameState>,
    pub out_tx: mpsc::Sender<Message>,
}
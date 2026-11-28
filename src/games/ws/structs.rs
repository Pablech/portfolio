use std::sync::Arc;
use serde::Deserialize;
use tokio::sync::{broadcast, mpsc};

use crate::games::{GameHub, LiveRoom, RoomHandle, ServerMsg};

#[derive(Deserialize)]
pub struct WsParams {
    pub token: Option<String>,
}

pub struct Session {
    pub hub: Arc<GameHub>,
    pub live: Arc<LiveRoom>,
    pub room: Arc<dyn RoomHandle>,
    pub room_id: String,
    pub tx: broadcast::Sender<ServerMsg>,
    pub out_tx: mpsc::Sender<String>,
    pub pong_tx: mpsc::Sender<Vec<u8>>,
    pub seat: usize,
}

pub struct SessionParams {
    pub room_id: String,
    pub token: String,
    pub seat: usize,
    pub joined: String,
    pub chat: bool,
}
use std::{sync::{Arc, atomic::AtomicUsize}, time::Duration};
use dashmap::DashMap;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::repositories::games::domain::{
    enums::{AgentKind, GameState}, 
    traits::{CpuBrain, RoomHandle, GameLogic},
};

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

pub struct Agent {
    pub kind: AgentKind,
    pub name: String,
    pub fig: char,
    pub token: Option<String>,
}

pub struct CreatedRoom {
    pub room_id: String,
    pub host_token: String,
    pub guest_token: Option<String>,
}

pub struct GameHub {
    pub rooms: DashMap<String, Arc<LiveRoom>>,
    pub reap_grace: Duration,
    pub orphan_ttl: Duration,
}

pub struct GameRoom<L: GameLogic> {
    pub room_id: String,
    pub game: L,
    pub agents: Vec<Agent>,
    pub turn: usize,
    pub finished: bool,
    pub brains: Vec<Option<Box<dyn CpuBrain<L>>>>,
    pub request_cache: LruCache<String, GameState>,
    pub rematch_votes: Vec<bool>,
    pub round: usize,
}

pub struct LiveRoom {
    pub handle: Arc<dyn RoomHandle>,
    pub tx: broadcast::Sender<GameState>,
    pub live: AtomicUsize,
}
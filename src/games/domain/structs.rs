use std::{sync::{Arc, atomic::{AtomicBool, AtomicUsize}}, time::{Duration, Instant}};
use dashmap::DashMap;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};
use axum::http::StatusCode;

use crate::games::{ServerMsg, domain::{
    enums::{AgentKind, GameState}, traits::{CpuBrain, GameLogic, RoomHandle},
}};

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

#[derive(Debug, Default)]
pub struct SmartishBrain;

pub struct Agent {
    pub kind: AgentKind,
    pub name: String,
    pub fig: char,
    pub token: Option<String>,
    pub default_name: String,
    pub default_fig: char,
}

pub struct CreatedRoom {
    pub room_id: String,
    pub host_token: String,
}

pub struct GameHub {
    pub rooms: DashMap<String, Arc<LiveRoom>>,
    pub reap_grace: Duration,
    pub orphan_ttl: Duration,
    pub seat_grace: Duration,
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
    pub last_chat: Vec<Option<Instant>>,
    pub last_left: Vec<Option<Instant>>,
    pub seat_grace: Duration,
}

pub struct LiveRoom {
    pub handle: Arc<dyn RoomHandle>,
    pub tx: broadcast::Sender<ServerMsg>,
    pub seats: DashMap<usize, mpsc::Sender<String>>,
    pub teams: DashMap<usize, usize>,
    pub live: AtomicUsize,
    pub ever_joined: AtomicBool,
}

pub struct WsError(pub StatusCode, pub String);
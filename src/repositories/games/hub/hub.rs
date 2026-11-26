use std::{num::NonZeroUsize, sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}}, time::Duration};
use dashmap::DashMap;
use lru::LruCache;
use tokio::sync::{Mutex, broadcast};

use crate::repositories::games::{Agent, AgentKind, CpuBrain, GameError, GameHub, GameLogic, GameRoom, LiveRoom, RoomHandle, generate_token,};

impl GameHub {
    pub fn new(rgs: u64, ots: u64, sgs: u64) -> Self {
        Self { 
            rooms: DashMap::new(), 
            reap_grace: Duration::from_secs(rgs * 60), 
            orphan_ttl: Duration::from_secs(ots * 60),
            seat_grace: Duration::from_secs(sgs * 60),
        }
    }

    pub fn get_room(&self, id: &str) -> Option<Arc<dyn RoomHandle>> {
        self.rooms.get(id).map(|r| r.handle.clone())
    }

    pub fn remove_room(&self, id: &str) {
        self.rooms.remove(id);
        tracing::debug!("sala {} removida", id);
    }

    pub fn enter(self: &Arc<Self>, id: &str) -> Option<Arc<LiveRoom>> {
        let r = self.rooms.get(id)?.clone();
        let prev = r.live.fetch_add(1, Ordering::SeqCst);
        if prev == 0 {
            r.ever_joined.store(true, Ordering::SeqCst);
        }
        Some(r)
    }

    pub fn exit(self: &Arc<Self>, id: &str, live: &Arc<LiveRoom>) {
        let prev = live.live.fetch_sub(1, Ordering::SeqCst); 
        if prev == 1 {
            self.reap_after(id.to_string(), live.clone(), self.reap_grace, "vazia após grace", false);
        }
    }

    pub fn create_room<L: GameLogic>(self: &Arc<Self>, agents: Vec<Agent>, brains: Vec<Option<Box<dyn CpuBrain<L>>>>)
        -> Result<(String, String), GameError> {

        let room_id = generate_token()[..8].to_string();
        let token = agents.first()
            .and_then(|a| a.token.clone())
            .ok_or_else(|| GameError::ErrorJoin("sala sem agents host com token".into()))?;

        let (tx, _rx) = broadcast::channel(16);
        let n_agents = agents.len();
        let rematch_votes: Vec<bool> = agents.iter()
            .map(|a| matches!(a.kind, AgentKind::Cpu))
            .collect();

        let room = {
            let mut room = GameRoom::<L> {
                room_id: room_id.clone(),
                game: L::new(),
                agents,
                turn: 0,
                finished: false,
                brains,
                request_cache: LruCache::new(NonZeroUsize::new(30).unwrap()),
                rematch_votes,
                round: 0,
                last_chat: vec![None; n_agents],
                last_left: vec![None; n_agents],
                seat_grace: self.seat_grace,
            };
            room.game.on_host_seated();
            if room.is_full() { room.game.on_table_full(); }
            room
        };

        let live = Arc::new(LiveRoom {
            handle: Arc::new(Mutex::new(room)),
            tx,
            seats: DashMap::new(),
            teams: DashMap::new(),
            live: AtomicUsize::new(0),
            ever_joined: AtomicBool::new(false),
        });

        self.rooms.insert(room_id.clone(), live.clone());
        self.reap_after(room_id.clone(), live.clone(), self.orphan_ttl, "órfã: nunca conectada", true);

        Ok((room_id, token))
    }

    fn reap_after(self: &Arc<Self>, room_id: String, live: Arc<LiveRoom>,
                delay: Duration, reason: &'static str, only_if_never_visited: bool) {

        let hub = self.clone();

        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let ceifou = hub.rooms.remove_if(&room_id, |_k, v| {
                Arc::ptr_eq(v, &live)
                    && v.live.load(Ordering::SeqCst) == 0
                    && (!only_if_never_visited || !v.ever_joined.load(Ordering::SeqCst))
            });
            if ceifou.is_some() {
                tracing::debug!("sala {} ceifada ({})", room_id, reason);
            }
        });
    }
}
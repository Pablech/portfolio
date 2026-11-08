use std::{num::NonZeroUsize, sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}}, time::Duration};
use dashmap::DashMap;
use lru::LruCache;
use tokio::sync::{Mutex, broadcast};
use anyhow::{Error, anyhow};

use crate::repositories::games::{
    AgentKind, GameHub, LiveRoom, MoveOutcome, 
    domain::{enums::{GameError, GameState, PlayResult}, 
    structs::{Agent, GameRoom}, 
    traits::{CpuBrain, GameLogic, RoomHandle}}, generate_token,
};

impl GameHub {
    pub fn new(rgs: u64, ots: u64) -> Self {
        Self { 
            rooms: DashMap::new(), 
            reap_grace: Duration::from_secs(rgs * 60), 
            orphan_ttl: Duration::from_secs(ots * 60),
        }
    }

    pub fn get_room(&self, id: &str) -> Option<Arc<dyn RoomHandle>> {
        self.rooms.get(id).map(|r| r.handle.clone())
    }

    pub fn remove_room(&self, id: &str) {
        self.rooms.remove(id);
        tracing::info!("sala {} removida", id);
    }

    pub fn enter(self: &Arc<Self>, id: &str) -> Option<Arc<LiveRoom>> {
        let r = self.rooms.get(id)?.clone();
        let prev = r.live.fetch_add(1, Ordering::SeqCst);
        if prev == 0 {
            r.ever_joined.store(true, Ordering::SeqCst);   // PRIMEIRA visita: aqui
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
        -> Result<(String, String), Error> {

        let room_id = generate_token()[..8].to_string();
        let token = agents.first()
            .and_then(|a| a.token.clone())
            .ok_or_else(|| anyhow!("sala sem agents host com token"))?;

        let (tx, _rx) = broadcast::channel(16);
        let n_agents = agents.len();
        let live = Arc::new(LiveRoom {
            handle: Arc::new(Mutex::new(GameRoom::<L> { 
                room_id: room_id.clone(), 
                game: L::new(), 
                agents, 
                turn: 0, 
                finished: false, 
                brains, 
                request_cache: LruCache::new(NonZeroUsize::new(30).unwrap()),
                rematch_votes: vec![false; n_agents],
                round: 0,
            })),
            tx,
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
                tracing::info!("sala {} ceifada ({})", room_id, reason);
            }
        });
    }
}

impl<L: GameLogic> GameRoom<L> {
    fn resolve_after_place(&mut self) -> Result<MoveOutcome, GameError> {
        loop {
            let fig = self.turn as u8 + 1;

            if self.game.check_win(fig) {
                self.finished = true;
                return Ok(MoveOutcome::Win { 
                    output: self.game.output(&self.agents), 
                    winner: self.agents[self.turn].name.clone(), 
                });
            }

            if self.game.check_draw() {
                self.finished = true;
                return Ok(MoveOutcome::Draw { 
                    output: self.game.output(&self.agents) 
                });
            }

            if self.brains[(self.turn + 1) % self.agents.len()].is_none() {
                self.turn = (self.turn + 1) % self.agents.len();
                break;
            }

            self.turn = (self.turn + 1) % self.agents.len();

            if let Some(brain) = self.brains[self.turn].take() {
                let fig = self.turn as u8 + 1;
                let legal = self.game.legal_moves();
                let mv = brain.choose(&self.game, &legal, fig);
                self.game.place(mv, fig)?;
                self.brains[self.turn] = Some(brain);
            }
        }

        Ok(MoveOutcome::Continue { 
            output: self.game.output(&self.agents), 
            next_player: self.agents[self.turn].name.clone(),
        })
    }

    fn reset(&mut self) -> MoveOutcome {
        self.game = L::new();
        self.finished = false;
        self.turn = self.round % self.agents.len();
        self.round += 1;
        self.rematch_votes.iter_mut().for_each(|v| *v = false);
        self.request_cache.clear();

        MoveOutcome::Continue { 
            output: self.game.output(&self.agents), 
            next_player: self.agents[self.turn].name.clone(), 
        }
    }
}

#[async_trait::async_trait]
impl <L: GameLogic> RoomHandle for Mutex<GameRoom<L>> {
    async fn join(&self, token: &str, req_id: Option<&str>) -> Result<GameState, GameError> {
        let mut room = self.lock().await;

        if let Some(req_id) = req_id {
            if let Some(cached_state) = room.request_cache.get(req_id) {
                return Ok(cached_state.clone());
            }
        }

        for a in &mut room.agents {
            if a.token.as_deref() == Some(token) {
                let res = GameState::JoinInfo { 
                    joined: a.name.clone(), 
                    room_id: room.room_id.clone()
                };
                if let Some(req_id) = req_id {
                    room.request_cache.put(req_id.to_string(), res.clone());
                } 
                return Ok(res);
            }
        }
        Err(GameError::ErrorJoin("token inválido para esta sala".into()))
    }

    async fn play(&self, token: &str, input: &str, req_id: Option<&str>) -> Result<PlayResult, GameError> {
        let mut room = self.lock().await;

        if let Some(req_id) =req_id {
            if let Some(cached) = room.request_cache.get(req_id) {
                return Ok(PlayResult::Replayed(cached.clone()));
            }
        }

        if room.finished {
            return Err(GameError::ErrorMove("partida encerrada".into()));
        }

        let agent_indx = room.agents.iter().position(|a| a.token.as_deref() == Some(token))
            .ok_or_else(|| GameError::ErrorJoin("token inválido".into()))?;

        if agent_indx != room.turn {
            return Err(GameError::ErrorMove("não é seu turno".into()));
        }

        let legal = room.game.legal_moves();
        let mv: usize = input.trim().parse::<usize>()
            .map_err(|_| GameError::ErrorMove(format!("entrada {} inválida", input)))?;

        let mv = mv.checked_sub(1)
            .ok_or_else(|| GameError::ErrorMove("coluna começa em 1".into()))?;

        if !legal.contains(&mv) {
            let col = mv + 1;
            let reason = if (0..7).contains(&mv) { "coluna cheia" } else { "coluna inválida" }; 
            return Err(GameError::ErrorMove(format!("coluna {}: {}", col,  reason)));
        }

        let player = room.turn as u8 + 1;
        room.game.place(mv, player)?;
        let outcome = room.resolve_after_place()?;

        if let Some(req_id) = req_id {
            room.request_cache.put(req_id.to_string(), GameState::State(outcome.clone()));
        }
        Ok(PlayResult::Fresh(GameState::State(outcome)))
    }

    async fn rematch(&self, token: &str, accept: bool, req_id: Option<&str>) -> Result<PlayResult, GameError> {
        let mut room = self.lock().await;

        if let Some(cached) = req_id.and_then(|req| room.request_cache.get(req)) {
            return Ok(PlayResult::Replayed(cached.clone()));
        }

        let idx = room.agents.iter().position(|a| a.token.as_deref() == Some(token))
            .ok_or_else(|| GameError::ErrorJoin("token inválido".into()))?;
        if !room.finished {
            return Err(GameError::ErrorMove("partida em andamento".into()));
        }

        room.rematch_votes[idx] = accept;
        let GameRoom {agents, rematch_votes, .. } = &mut *room;
        for (a, voted) in agents.iter().zip(rematch_votes.iter_mut()) {
            if matches!(a.kind, AgentKind::Cpu) { *voted = true; }
        }

        let (state, kill) = if !accept {
            (GameState::State(MoveOutcome::RematchDeclined {
                by: room.agents[idx].name.clone(),
            }), true)
        } else {
            let all_in = room.agents.iter().zip(room.rematch_votes.iter())
                .all(|(a, voted)| matches!(a.kind, AgentKind::Cpu) || *voted);
            if all_in {
                (GameState::State(room.reset()), false)
            } else {
                (GameState::State(MoveOutcome::RematchPending {
                    who: room.agents[idx].name.clone(),
                }), false)
            }
        };

        if let Some(req) = req_id {
            room.request_cache.put(req.to_string(), state.clone());
        }

        Ok(if kill {
            PlayResult::Dead(state)
        } else {
            PlayResult::Fresh(state)
        })
    }
}
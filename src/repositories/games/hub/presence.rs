use std::time::{Duration, Instant};

use crate::{handlers::structs::ClaimedSeat, repositories::games::{AgentKind, GameError, GameLogic, GameRoom, GameState, generate_token, utils::{choice_fig, resolve_name}}};

impl <L: GameLogic> GameRoom<L> {
    pub(super) fn seat_info(&mut self, token: &str, req_id: Option<&str>) -> Result<GameState, GameError> {
        if let Some(req) = req_id {
            if let Some(cached) = self.request_cache.get(req) {
                return Ok(cached.clone());
            }
        }

        let Some(indx) = self.agents.iter().position(|a| a.token.as_deref() == Some(token)) else {
            return Err(GameError::ErrorJoin("token inválido para esta sala".into()));
        };

        self.last_left[indx] = None;

        let humans = self.agents.iter().filter(|a| matches!(a.kind, AgentKind::Human)).count();
        let res = GameState::JoinInfo { 
            joined: self.agents[indx].name.clone(), 
            room_id: self.room_id.clone(), 
            chat: humans >= 2,
            seat: indx,
        };

        if let Some(req) = req_id {
            self.request_cache.put(req.to_string(), res.clone());
        } 
        Ok(res)
    }

    pub(super) fn claim_seat(&mut self, name: Option<String>, fig: Option<char>) -> Result<ClaimedSeat, GameError> {
        let indx = self.agents.iter().enumerate().position(|(i, a)| {
            matches!(a.kind, AgentKind::Human) && (a.token.is_none() || 
                self.last_left[i].map_or(false, |at| at.elapsed() >= self.seat_grace))
            }).ok_or_else(|| GameError::ErrorJoin("sala cheia ou partida solo".into()))?;
        
        let taken_names: Vec<String> = self.agents.iter()
            .enumerate().filter(|&(i, _)| i != indx)
            .map(|(_, a)| a.name.clone()).collect();

        let default_name = self.agents[indx].default_name.clone();
        let resolved_name = resolve_name(name, &default_name, indx, &taken_names);
        
        let taken_figs: Vec<char> = self.agents.iter().map(|a| a.fig).collect();
        let resolved_fig = fig.unwrap_or_else(|| choice_fig(&taken_figs));
        
        if self.agents.iter().enumerate().any(|(i, a)| i != indx && a.fig == resolved_fig) {
            return Err(GameError::ErrorValidation(format!("figura {} em uso", resolved_fig)));
        }

        let token = generate_token();
        self.last_left[indx] = None;

        {
            let agent = &mut self.agents[indx];
            agent.token = Some(token.clone());
            agent.name = resolved_name;
            agent.fig = resolved_fig;
            self.rematch_votes[indx] = false;
        }

        if self.finished && self.is_full() { self.reset(); }

        let agent = &self.agents[indx];
        Ok(ClaimedSeat { token, joined: agent.name.clone(), room_id: self.room_id.clone() })
    }

    pub(super) fn mark_left(&mut self, token: &str) -> Result<String, GameError> {
        let indx = self.get_index(token)
            .ok_or_else(|| GameError::ErrorJoin("token inválido".into()))?;

        self.last_left[indx] = Some(Instant::now());
        Ok(self.agents[indx].name.clone())
    }

    pub(super) fn vacate_seat(&mut self, token: &str) -> Result<String, GameError> {
        let indx = self.get_index(token)
            .ok_or_else(|| GameError::ErrorJoin("token inválido".into()))?;

        self.agents[indx].token = None;
        self.last_left[indx] = None;
        Ok(self.agents[indx].name.clone())
    }

    pub(super) fn chat_line(&mut self, token: &str, text: &str) -> Result<String, GameError> {
        let indx = self.get_index(token)
            .ok_or_else(|| GameError::ErrorJoin("token inválido".into()))?;

        let trimmed = text.trim();
        if trimmed.is_empty() {
            return  Err(GameError::ErrorValidation("mensagem vazia".into()));
        }

        if trimmed.chars().count() > 240 {
            return Err(GameError::ErrorValidation("mensagem muito longa (máximo 240 caracteres)".into()));
        }

        if let Some(Some(at)) = self.last_chat.get(indx) {
            if at.elapsed() < Duration::from_millis(500) { 
                return Err(GameError::ErrorValidation(format!("rate limit de: {}", self.agents[indx].name)));
            }
        }

        self.last_chat[indx] = Some(Instant::now());
        Ok(self.agents[indx].name.clone())
    }

    pub(super) fn outputs(&self) -> Vec<String> {
        if self.game.shared_output() {
            vec![self.game.output(&self.agents, &None)]
        } else {
            (0..self.agents.len())
                .map(|seat| self.game.output(&self.agents, &Some(seat))).collect()
        }
    }

    pub(super) fn is_full(&self) -> bool {
        self.agents.iter().all(|a| matches!(a.kind, AgentKind::Cpu) || a.token.is_some())
    }
}
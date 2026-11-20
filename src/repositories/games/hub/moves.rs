use crate::repositories::games::{AgentKind, GameError, GameLogic, GameRoom, GameState, MoveOutcome, PlayResult};

impl <L: GameLogic> GameRoom<L> {
    pub(super) fn aply_moves(&mut self, token: &str, input: &str, req_id: Option<&str>) -> Result<PlayResult, GameError> {
        if let Some(cached) = req_id.and_then(|req| self.request_cache.get(req)) {
            return Ok(PlayResult::Replayed(cached.clone()));
        }

        if self.finished {
            return Err(GameError::ErrorMove("partida encerrada - rematch para jogar novamente".into()));
        }

        let agent_indx = self.get_index(token)
            .ok_or_else(|| GameError::ErrorJoin("token inválido".into()))?;

        if agent_indx != self.turn {
            return Err(GameError::ErrorMove("não é seu turno".into()));
        }

        let legal = self.game.legal_moves();

        let mv: usize = input.trim().parse::<usize>()
            .map_err(|_| GameError::ErrorMove(format!("entrada {} inválida", input)))?;

        let mv = mv.checked_sub(1)
            .ok_or_else(|| GameError::ErrorMove("coluna começa em 1".into()))?;

        if !legal.contains(&mv) {
            return Err(GameError::ErrorMove(format!("jogada {} indisponível", mv + 1)));
        }

        let player = self.turn as u8 + 1;
        self.game.place(mv, player)?;
        let outcome = self.resolve_after_place()?;

        if let Some(req) = req_id {
            self.request_cache.put(req.to_string(), GameState::State(outcome.clone()));
        } 

        Ok(PlayResult::Fresh(GameState::State(outcome)))
    }

    pub(super) fn vote_rematch(&mut self, token: &str, accept: bool, req_id: Option<&str>) -> Result<PlayResult, GameError> {
        if let Some(cached) = req_id.and_then(|req| self.request_cache.get(req)) {
            return Ok(PlayResult::Replayed(cached.clone()));
        }

        let agent_indx = self.get_index(token)
            .ok_or_else(|| GameError::ErrorJoin("token inválido".into()))?;

        if !self.finished {
            return Err(GameError::ErrorMove("partida em andamento".into()));
        }

        self.rematch_votes[agent_indx] = accept;

        let GameRoom { agents, rematch_votes, .. } = &mut *self;
        for (a, voted) in agents.iter().zip(rematch_votes.iter_mut()) {
            if matches!(a.kind, AgentKind::Cpu) { *voted = true; }
        }

        
        let all_in = self.agents.iter().zip(self.rematch_votes.iter())
            .all(|(a, voted)| matches!(a.kind, AgentKind::Cpu) || *voted);

        let state = if all_in {
            GameState::State(self.reset())
        } else {
            let who = self.agents.iter().zip(self.rematch_votes.iter())
                .find(|(a, voted)| matches!(a.kind, AgentKind::Human) && !**voted)
                .map(|(a, _)| a.name.clone())
                .expect("ramo pending implica humano sem voto");
            GameState::State(MoveOutcome::RematchPending { who, })
        };


        if let Some(req) = req_id {
            self.request_cache.put(req.to_string(), state.clone());
        }

        Ok(PlayResult::Fresh(state))
    }

    pub(super) fn snapshot_state(&self) -> GameState {
        GameState::State(MoveOutcome::Continue { 
            output: self.outputs(), 
            next_player: self.agents[self.turn].name.clone() 
        })
    }

    pub(super) fn rematch_round(&self) -> u64 {
        self.round as u64
    }

    pub(super) fn rematch_timeout(&mut self, cycle: u64) -> Option<(usize, String, String, GameState)> {
        if self.round as u64 != cycle { return None; }
        if !self.finished || !self.rematch_votes.iter().any(|&v| v) { return None; }

        let missing = self.agents.iter().zip(self.rematch_votes.iter())
            .position(|(a, voted)| matches!(a.kind, AgentKind::Human) 
                && !*voted && a.token.is_some())?;

        let name = self.agents[missing].name.clone();
        let state = GameState::State(MoveOutcome::RematchDeclined { by: name.clone() });
        let token = self.agents[missing].token.clone();
        Some((missing, name, token?, state))
    }

    fn init_rematch_votes(&mut self) {
        for (a, voted) in self.agents.iter().zip(self.rematch_votes.iter_mut()) {
            *voted = matches!(a.kind, AgentKind::Cpu);
        }
    }

    fn resolve_after_place(&mut self) -> Result<MoveOutcome, GameError> {
        loop {
            let fig = self.turn as u8 + 1;

            if self.game.check_win(fig) {
                self.finished = true;
                return Ok(MoveOutcome::Win { 
                    output: self.outputs(), 
                    winner: self.agents[self.turn].name.clone(), 
                });
            }

            if self.game.check_draw() {
                self.finished = true;
                return Ok(MoveOutcome::Draw { 
                    output: self.outputs() 
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
            output: self.outputs(), 
            next_player: self.agents[self.turn].name.clone(),
        })
    }

    pub(super) fn reset(&mut self) -> MoveOutcome {
        self.game = L::new();
        self.finished = false;
        self.turn = 0; // dice(self.agents.len());
        self.round += 1;
        self.init_rematch_votes();
        self.request_cache.clear();

        MoveOutcome::Continue { 
            output: self.outputs(), 
            next_player: self.agents[self.turn].name.clone(), 
        }
    }
}
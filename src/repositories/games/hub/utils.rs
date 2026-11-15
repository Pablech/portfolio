use crate::repositories::games::{GameLogic, GameRoom};

impl <L: GameLogic> GameRoom<L> {
    pub(super) fn get_index(&self, token: &str) -> Option<usize> {
         self.agents.iter().position(|a| a.token.as_deref() == Some(token))
    }
}
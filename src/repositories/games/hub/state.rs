use crate::repositories::games::{GameState, MoveOutcome};

impl GameState {
    pub fn for_seat(&self, seat: usize) -> GameState {
        let pick = |outs: &Vec<String>| {
            let view = if outs.len() == 1 {
                outs[0].clone()
            } else {
                outs.get(seat).cloned().unwrap_or_default()
            };
            vec![view]
        };

        match self {
            GameState::State(MoveOutcome::Continue { output, next_player }) => {
                GameState::State(MoveOutcome::Continue { output: pick(output), next_player: next_player.clone() })
            }
            GameState::State(MoveOutcome::Win { output, winner }) => {
                GameState::State(MoveOutcome::Win { output: pick(output), winner: winner.clone() })
            }
            GameState::State(MoveOutcome::Draw { output }) => {
                GameState::State(MoveOutcome::Draw { output: pick(output) })
            }
            _ => self.clone(),
        }
    }
}
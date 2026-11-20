use tokio::sync::mpsc;

use crate::repositories::games::{GameState, LiveRoom, ServerMsg};

impl LiveRoom {
    pub fn attach_seat(&self, seat: usize, tx: mpsc::Sender<String>) {
        self.seats.insert(seat, tx);
    }

    pub fn detach_seat(&self, seat: usize) {
        self.seats.remove(&seat);
    }

    pub async fn broadcast_state(&self, state: &GameState, actor: Option<(usize, Option<String>)>) {
        for (seat, tx) in self.seat_targets(usize::MAX) { 
            let req_id = match actor {
                Some((actor_seat, ref rid)) if actor_seat == seat => rid.clone(),
                _ => None,
            };
            if let Ok(json) = serde_json::to_string(&ServerMsg::State {
                state: state.for_seat(seat), req_id,
            }) {
                let _ = tx.send(json).await;
            }
        }
    }

    pub async fn send_to_seat(&self, seat: usize, msg: &ServerMsg) {
        if let Some(tx) = self.seats.get(&seat) {
            let tx = tx.value().clone();
            if let Ok(json) = serde_json::to_string(msg) {
                let _ = tx.send(json).await;
            } 
        } 
    }

    pub fn seat_targets(&self, exclude: usize) -> Vec<(usize, mpsc::Sender<String>)> {
        self.seats.iter()
            .filter(|e| *e.key() != exclude)
            .map(|e| (*e.key(), e.value().clone()))
            .collect()
    }
}
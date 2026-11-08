use std::sync::Arc;
use askama_axum::IntoResponse;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Path, Query, State},
    http::StatusCode,
    response::Response,
};
use futures::{SinkExt, StreamExt}; 
use tokio::sync::{broadcast, mpsc};

use crate::{
    repositories::games::{
        ClientMsg, Flow, Frame, GameError, GameHub, GameState, LiveRoom, PlayResult, ServerMsg, Session, WsParams
    }, state::AppState
};

fn ser(msg: &ServerMsg) -> String { serde_json::to_string(msg).expect("ServerMsg sempre serializa") }
fn ser_state(st: &GameState) -> String { serde_json::to_string(st).expect("GameState sempre serializa") }
fn err_text(e: &GameError) -> String { format!("{:?}", e) }
fn de<T: serde::de::DeserializeOwned>(txt: &str) -> Option<T> { serde_json::from_str(txt).ok() }

pub async fn handler(State(state): State<AppState>,
    Path(room_id): Path<String>, Query(params): Query<WsParams>,
    ws: WebSocketUpgrade, ) -> Response {
        
    let Some(live) = state.games.enter(&room_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if let Some(ref token) = params.token {
        if live.handle.join(token, None).await.is_err() {
            state.games.exit(&room_id, &live);
            return StatusCode::UNAUTHORIZED.into_response();
        }
    }

    let rx = live.tx.subscribe();
    let hub = state.games.clone();

    ws.on_upgrade(move |socket| Session::connection(hub, live, room_id, params.token, rx, socket))
}

impl Session {
    async fn send(&self, json: String) {
        let _ = self.out_tx.send(Message::Text(json)).await;
    }

    async fn push_error(&self, message: String, fatal: bool) {
        self.send(ser(&ServerMsg::Error { message, fatal })).await;
    }

    async fn push_state(&self, state: GameState, req_id: Option<String>) {
        self.send(ser(&ServerMsg::State { state, req_id })).await;
    }

    async fn broadcast(&self, state: &GameState) {
        let _ = self.tx.send(state.clone());
    }

    async fn on_frame(&self, msg: Option<Result<Message, axum::Error>>) -> Frame {
        match msg {
            Some(Ok(Message::Text(txt))) => Frame::Text(txt),
            Some(Ok(Message::Ping(p))) => {
                let _ = self.out_tx.send(Message::Pong(p)).await;
                Frame::Skip
            }
            Some(Ok(Message::Pong(_))) => Frame::Skip,
            Some(Ok(Message::Binary(_))) => {
                self.push_error("protocolo aceita apenas texto".into(), false).await;
                Frame::Skip
            }
            Some(Ok(Message::Close(_))) | None | Some(Err(_)) => Frame::Close,
        }
    }

    async fn on_broadcast(&self, gs: GameState, skip_echo: &mut Option<String>) {
        let state_json = ser_state(&gs);
        if skip_echo.as_deref() == Some(state_json.as_str()) {
            *skip_echo = None;
        } else {
            self.push_state(gs, None).await;
        }
    }

    async fn dispatch(&self, msg: ClientMsg, skip_echo: &mut Option<String>) -> Flow {
        match msg {
            ClientMsg::Join { token, req_id } => self.on_join(&token, req_id).await,
            ClientMsg::Play { token, input, req_id } => {
                self.on_play(&token, &input, req_id, skip_echo).await
            }
            ClientMsg::Rematch { token, accept, req_id } => {
                self.on_rematch(&token, accept, req_id, skip_echo).await
            }
        }
    }

    async fn on_join(&self, token: &str, req_id: Option<String>) -> Flow {
        match self.room.join(token, req_id.as_deref()).await {
            Ok(GameState::JoinInfo { joined, .. }) => {
                self.send(ser(&ServerMsg::Joined { name: joined, room_id: self.room_id.clone(), })).await;
            }
            Ok(_) => unreachable!("join só devolve JoinInfo"),
            Err(e) => self.push_error(err_text(&e), false).await,
        }
        Flow::Continue
    }

    async fn on_play(&self, token: &str, input: &str, req_id: Option<String>, skip_echo: &mut Option<String>,) -> Flow {
        match self.room.play(token, input, req_id.as_deref()).await {
            Ok(PlayResult::Fresh(st)) => {
                self.broadcast(&st).await;
                *skip_echo = Some(ser_state(&st));
                self.push_state(st, req_id).await;
            }
            Ok(PlayResult::Replayed(st)) => {
                self.push_state(st, req_id).await;
            }
            Ok(PlayResult::Dead(_)) => {
                unreachable!("play nunca devolve Dead; Dead é exclusivo do rematch");
            }
            Err(e) => self.push_error(err_text(&e), false).await,
        }
        Flow::Continue
    }

    async fn on_rematch(&self, token: &str, accept: bool, req_id: Option<String>, skip_echo: &mut Option<String>) -> Flow {
        match self.room.rematch(token, accept, req_id.as_deref()).await {
            Ok(PlayResult::Fresh(st)) => {
                self.broadcast(&st).await;
                *skip_echo = Some(ser_state(&st));
                self.push_state(st, req_id).await;
            }
            Ok(PlayResult::Replayed(st)) => {
                self.push_state(st, req_id).await;
            }
            Ok(PlayResult::Dead(st)) => {
                self.broadcast(&st).await;
                self.push_state(st, req_id).await;
                self.hub.remove_room(&self.room_id);
                return Flow::Close;
            }
            Err(e) => self.push_error(err_text(&e), false).await,
        }
        Flow::Continue
    }

    async fn connection(hub: Arc<GameHub>, live: Arc<LiveRoom>, room_id: String, token: Option<String>,
        mut rx: broadcast::Receiver<GameState>, socket: WebSocket) {

        let (mut sink, mut stream) = socket.split();

        if let Some(ref tok) = token {
            if let Ok(GameState::JoinInfo { joined, .. }) = live.handle.join(tok, None).await {
                let _ = sink.send(Message::Text(ser(&ServerMsg::Joined { name: joined, room_id: room_id.clone(), }))).await;
            }
        }

        let (out_tx, mut out_rx) = mpsc::channel::<Message>(16);
        let writer = tokio::spawn(async move {
            while let Some(msg) = out_rx.recv().await {
                if sink.send(msg).await.is_err() { break; }
            }
        });

        let sess = Session {
            room: live.handle.clone(),
            tx: live.tx.clone(),
            hub, live, room_id, out_tx,
        };

        let mut skip_echo: Option<String> = None;

        loop {
            tokio::select! {
                st = rx.recv() => {
                    match st {
                        Ok(gs) => sess.on_broadcast(gs, &mut skip_echo).await,
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            sess.push_error(format!("atrasou {} estados", n), false).await;
                        }
                        Err(_) => break,
                    }
                }
                msg = stream.next() => {
                    let txt = match sess.on_frame(msg).await {
                        Frame::Text(txt) => txt,
                        Frame::Skip => continue,
                        Frame::Close => break,
                    };
                    let Some(client_msg) = de::<ClientMsg>(&txt) else {
                        sess.push_error("mensagem inválida".into(), false).await;
                        continue;
                    };
                    if sess.dispatch(client_msg, &mut skip_echo).await == Flow::Close { break; }
                }
            }
        }
        sess.hub.exit(&sess.room_id, &sess.live);
        writer.abort();
    }
}

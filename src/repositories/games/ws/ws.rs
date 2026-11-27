use std::{sync::Arc, time::Duration};
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
        ClientMsg, Flow, Frame, GameError, GameHub, GameState, LiveRoom, 
        MoveOutcome, PlayResult, ServerMsg, Session, WsParams,
    }, state::AppState
};

fn ser(msg: &ServerMsg) -> String { serde_json::to_string(msg).expect("ServerMsg sempre serializa") }
fn de<T: serde::de::DeserializeOwned>(txt: &str) -> Option<T> { serde_json::from_str(txt).ok() }

pub async fn handler(State(state): State<AppState>,
    Path(room_id): Path<String>, Query(params): Query<WsParams>,
    ws: WebSocketUpgrade, ) -> Response {
        
    let Some(live) = state.games.enter(&room_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Some(token) = params.token else {
        state.games.exit(&room_id, &live);
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let Ok(GameState::JoinInfo { joined, chat, seat, .. }) = live.handle.join(&token, None).await else {
        state.games.exit(&room_id, &live);
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let rx = live.tx.subscribe();
    let hub = state.games.clone();

    ws.on_failed_upgrade({
        let hub = hub.clone();
        let live = live.clone();
        let room_id = room_id.clone();
        move |_err| hub.exit(&room_id, &live)
    }).on_upgrade(move |socket| {
        Session::connection(hub, live, room_id, token, seat, joined, chat, rx, socket)
    })
}

impl Session {
    async fn send(&self, json: String) {
        let _ = self.out_tx.send(json).await;
    }

    async fn push_error(&self, err: &GameError) {
        let (code, err) = err.as_tuple();
        self.send(ser(&ServerMsg::Error { code, message: err.to_string() })).await;
    }

    async fn push_state(&self, state: GameState, req_id: Option<String>) {
        self.send(ser(&ServerMsg::State { state, req_id })).await;
    }

    async fn on_frame(&self, msg: Option<Result<Message, axum::Error>>) -> Frame {
        match msg {
            Some(Ok(Message::Text(txt))) => Frame::Text(txt),
            Some(Ok(Message::Ping(p))) => {
                let _ = self.pong_tx.send(p.to_vec()).await;
                Frame::Skip
            }
            Some(Ok(Message::Pong(_))) => Frame::Skip,
            Some(Ok(Message::Binary(_))) => {
                self.push_error(
                    &GameError::ErrorProtocol("protocolo aceita apenas texto".into())
                ).await;
                Frame::Skip
            }
            Some(Ok(Message::Close(_))) | None | Some(Err(_)) => Frame::Close,
        }
    }

    async fn on_broadcast(&self, msg: ServerMsg) {
        self.send(ser(&msg)).await;
    }

    async fn dispatch(&self, msg: ClientMsg) -> Flow {
        match msg {
            ClientMsg::Join { token, req_id } => self.on_join(&token, req_id).await,
            ClientMsg::Play { token, input, req_id } => {
                self.on_play(&token, &input, req_id).await
            }
            ClientMsg::Rematch { token, accept, req_id } => {
                self.on_rematch(&token, accept, req_id).await
            }
            ClientMsg::Chat { token, text, ..} => {
                match self.room.chat(&token, &text).await {
                    Ok(author) => {
                        { let _ = self.tx.send(ServerMsg::Chat { author, text: text.trim().to_owned() }); }
                    }
                    Err(e) => self.push_error(&e).await,
                }
                Flow::Continue
            }
            ClientMsg::Quit { token, .. } => {
                match self.room.quit(&token).await {
                    Ok(name) => {
                        let _ = self.tx.send(ServerMsg::Left { name: name.clone() });
                        self.send(ser(&ServerMsg::Quit { name })).await;
                        return Flow::Close;
                    }
                    Err(e) => self.push_error(&e).await,
                }
                Flow::Continue
            }
        }
    }

    async fn on_join(&self, token: &str, req_id: Option<String>) -> Flow {
        match self.room.join(token, req_id.as_deref()).await {
            Ok(GameState::JoinInfo { joined, chat, .. }) => {
                self.send(ser(&ServerMsg::Joined { name: joined, room_id: self.room_id.clone(), chat })).await;
            }
            Ok(_) => unreachable!("join só devolve JoinInfo"),
            Err(e) => self.push_error(&e).await,
        }
        Flow::Continue
    }

    async fn on_play(&self, token: &str, input: &str, req_id: Option<String>,) -> Flow {
        match self.room.play(token, input, req_id.as_deref()).await {
            Ok(PlayResult::Fresh(st)) => {
                self.live.broadcast_state(&st, Some((self.seat, req_id))).await;
            }
            Ok(PlayResult::Replayed(st)) => {
                self.push_state(st.for_seat(self.seat), req_id).await;
            }
            Err(e) => self.push_error(&e).await,
        }
        Flow::Continue
    }

    async fn on_rematch(&self, token: &str, accept: bool, req_id: Option<String>) -> Flow {
        match self.room.rematch(token, accept, req_id.as_deref()).await {
            Ok(PlayResult::Fresh(st)) => {
                if matches!(st, GameState::State(MoveOutcome::RematchPending { .. })) {
                    let room = self.room.clone();
                    let live  = self.live.clone();
                    let cycle = room.rematch_cycle().await;

                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(60)).await;
                        if let Some((kicked, name, token, st)) = room.timeout_rematch(cycle).await {
                            live.broadcast_state(&st, None).await;
                            let _ = live.tx.send(ServerMsg::Left { name: name.clone() });
                            let _ = room.quit(&token).await;
                            live.send_to_seat(kicked, &ServerMsg::Quit { name }).await;
                            live.detach_seat(kicked);
                        }
                    });
                }
                
                self.live.broadcast_state(&st, Some((self.seat, req_id))).await;
            }
            Ok(PlayResult::Replayed(st)) => {
                self.push_state(st.for_seat(self.seat), req_id).await;
            }
            Err(e) => self.push_error(&e).await,
        }
        Flow::Continue
    }
    
    async fn connection(hub: Arc<GameHub>, live: Arc<LiveRoom>, room_id: String, token: String, seat: usize,
        joined: String, chat: bool, mut rx: broadcast::Receiver<ServerMsg>, socket: WebSocket) {

        let (mut sink, mut stream) = socket.split();

        let (out_tx, mut out_rx) = mpsc::channel::<String>(16);
        let (pong_tx, mut pong_rx) = mpsc::channel::<Vec<u8>>(4);

        let writer = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(json) = out_rx.recv() => {
                        if sink.send(Message::Text(json)).await.is_err() { break; }
                    }
                    Some(payload) = pong_rx.recv() => {
                        if sink.send(Message::Pong(payload)).await.is_err() { break; }
                    }
                    else => break,
                }
            }
            let _ = sink.close().await;
        });

        live.attach_seat(seat, out_tx.clone());
        
        let _ = out_tx.send(ser(&ServerMsg::Joined { name: joined.clone(), room_id: room_id.clone(), chat })).await;

        let st = live.handle.snapshot().await;
        let _ = out_tx.send(ser(&ServerMsg::State { state: st.for_seat(seat), req_id: None })).await;

        for (other_seat, tx) in live.seat_targets(seat) {
            let _ = tx.send(ser(&ServerMsg::Joined { name: joined.clone(), room_id: room_id.clone(), chat })).await;
            let _ = tx.send(ser(&ServerMsg::State { state: st.for_seat(other_seat), req_id: None })).await;
        }

        let sess = Session {
            room: live.handle.clone(),
            tx: live.tx.clone(),
            hub, live, room_id, 
            out_tx, pong_tx, seat,
        };

        loop {
            tokio::select! {
                st = rx.recv() => {
                    match st {
                        Ok(gs) => sess.on_broadcast(gs).await,
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            sess.push_error(
                                &GameError::ErrorProtocol(format!("atrasou {} estados", n))
                            ).await;
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
                        sess.push_error(
                            &GameError::ErrorProtocol("mensagem inválida".into())
                        ).await;
                        continue;
                    };
                    if sess.dispatch(client_msg).await == Flow::Close { break; }
                }
            }
        }

        if let Ok(name) = sess.live.handle.leave(&token).await {
            let _ = sess.live.tx.send(ServerMsg::Left { name: name.to_string() });
        }

        let hub = sess.hub.clone();
        let (room_id, live) = (sess.room_id.clone(), sess.live.clone());
        sess.live.detach_seat(sess.seat);
        drop(sess);
        let _ = writer.await;
        hub.exit(&room_id, &live);
    }
}

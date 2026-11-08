use std::sync::Arc;
use askama_axum::IntoResponse;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Path, Query, State},
    http::StatusCode,
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::broadcast;

use crate::{
    repositories::games::{
        ClientMsg, GameError, GameState, PlayResult, RoomHandle, ServerMsg
    }, 
    state::AppState
};



#[derive(Deserialize)]
pub struct WsParams {
    pub token: Option<String>,
}

fn ser(msg: &ServerMsg) -> String { serde_json::to_string(msg).expect("ServerMsg sempre serializa") }
fn ser_state(st: &GameState) -> String { serde_json::to_string(st).expect("GameState sempre serializa") }
fn err_text(e: &GameError) -> String { format!("{:?}", e) }
fn de<T: serde::de::DeserializeOwned>(txt: &str) -> Option<T> { serde_json::from_str(txt).ok() }

pub async fn handler(State(state): State<AppState>, Path(room_id): Path<String>,
    Query(params): Query<WsParams>, ws: WebSocketUpgrade, ) -> Response {

    let Some(room) = state.games.get_room(&room_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if let Some(ref token) = params.token {
        if room.join(token, None).await.is_err() {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    }

    let tx = state.games.channel_for(&room_id, 16);
    let rx = tx.subscribe();

    ws.on_upgrade(move |socket| connection(room, room_id, params.token, tx, rx, socket))
}

async fn connection(room: Arc<dyn RoomHandle>, room_id: String, token: Option<String>,
    tx: broadcast::Sender<GameState>, mut rx: broadcast::Receiver<GameState>, socket: WebSocket, ) {

    let (mut sink, mut stream) = socket.split();

    if let Some(ref tok) = token {
        if let Ok(GameState::JoinInfo { joined, .. }) = room.join(tok, None).await {
            let _ = sink.send(Message::Text(ser(&ServerMsg::Joined {
                name: joined, room_id: room_id.clone(),
            }))).await;
        }
    }

    let (out_tx, mut out_rx) = tokio::sync::mpsc::channel::<Message>(16);
    let writer = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if sink.send(msg).await.is_err() { break; }
        }
    });

    let mut skip_echo: Option<String> = None;

    loop {
        tokio::select! {
            st = rx.recv() => {
                match st {
                    Ok(gs) => {
                        let state_json = ser_state(&gs);
                        if skip_echo.as_deref() == Some(state_json.as_str()) {
                            skip_echo = None;
                        } else {
                            let _ = out_tx.send(Message::Text(ser(&ServerMsg::State {
                                state: gs, req_id: None,
                            }))).await;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        let _ = out_tx.send(Message::Text(ser(&ServerMsg::Error {
                            message: format!("atrasou {} estados", n), fatal: false,
                        }))).await;
                    }
                    Err(_) => break,
                }
            }

            msg = stream.next() => {
                let txt = match msg {
                    Some(Ok(Message::Text(txt))) => txt,
                    Some(Ok(Message::Ping(p))) => {
                        let _ = out_tx.send(Message::Pong(p)).await;
                        continue;
                    }
                    Some(Ok(Message::Pong(_))) => continue,
                    Some(Ok(Message::Binary(_))) => {
                        let _ = out_tx.send(Message::Text(ser(&ServerMsg::Error {
                            message: "protocolo aceita apenas texto".into(), fatal: false,
                        }))).await;
                        continue;
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                };

                let Some(client_msg) = de::<ClientMsg>(&txt) else {
                    let _ = out_tx.send(Message::Text(ser(&ServerMsg::Error {
                        message: "mensagem inválida".into(), fatal: false,
                    }))).await;
                    continue;
                };

                match client_msg {
                    ClientMsg::Join { token, req_id } => {
                        match room.join(&token, req_id.as_deref()).await {
                            Ok(GameState::JoinInfo { joined, .. }) => {
                                let _ = out_tx.send(Message::Text(ser(&ServerMsg::Joined {
                                    name: joined, room_id: room_id.clone(),
                                }))).await;
                            }
                            Ok(_) => unreachable!("join só devolve JoinInfo"),
                            Err(e) => {
                                let _ = out_tx.send(Message::Text(ser(&ServerMsg::Error {
                                    message: err_text(&e), fatal: false,
                                }))).await;
                            }
                        }
                    }

                    ClientMsg::Play { token, input, req_id } => {
                        match room.play(&token, &input, req_id.as_deref()).await {
                            Ok(PlayResult::Fresh(st)) => {
                                let _ = tx.send(st.clone());
                                skip_echo = Some(ser_state(&st));
                                let json = ser(&ServerMsg::State { state: st, req_id });
                                let _ = out_tx.send(Message::Text(json)).await;
                            }
                            Ok(PlayResult::Replayed(st)) => {
                                let _ = out_tx.send(Message::Text(ser(&ServerMsg::State {
                                    state: st, req_id,
                                }))).await;
                            }
                            Err(e) => {
                                let _ = out_tx.send(Message::Text(ser(&ServerMsg::Error {
                                    message: err_text(&e), fatal: false,
                                }))).await;
                            }
                        }
                    }
                }
            }
        }
    }

    writer.abort();
}
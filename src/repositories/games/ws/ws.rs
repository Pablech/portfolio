use std::sync::Arc;

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Path, State},
    response::Response,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;

use crate::{
    state::AppState,
    repositories::games::{GameError, GameHub, GameState, ws::{ClientMsg, ServerMsg}},
};

fn ser(msg: &ServerMsg) -> String { serde_json::to_string(msg).unwrap() }
fn err_text(e: &GameError) -> String { format!("{:?}", e) }
fn de<T: serde::de::DeserializeOwned>(txt: &str) -> Option<T> { serde_json::from_str(txt).ok() }

pub async fn handler(State(state): State<AppState>,
        Path(room_id): Path<String>, ws: WebSocketUpgrade, ) -> Response {
    ws.on_upgrade(move |socket| connection(state.games.clone(), room_id, socket))
}

async fn connection(hub: Arc<GameHub>, room_id: String, socket: WebSocket) {
    let (mut sink, mut stream) = socket.split();

    let Some(room) = hub.rooms.get(&room_id).map(|r| r.value().clone()) else {
        let _ = sink.send(Message::Text(ser(&ServerMsg::Error {
            message: "sala não encontrada".into(), fatal: true,
        }))).await;
        return;
    };

    let tx = hub.channel_for(&room_id, 16);
    let mut rx = tx.subscribe();
    let (out_tx, mut out_rx) = tokio::sync::mpsc::channel::<String>(16);
    let writer = tokio::spawn(async move {
        while let Some(json) = out_rx.recv().await {
            if sink.send(Message::Text(json)).await.is_err() { break; }
        }
    });

    let mut skip_echo: Option<String> = None;

    loop {
        tokio::select! {
            st = rx.recv() => {
                match st {
                    Ok(gs) => {
                        let json = ser(&ServerMsg::State { state: gs });
                        if skip_echo.as_deref() == Some(json.as_str()) {
                            skip_echo = None;
                        } else {
                            let _ = out_tx.send(json).await;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        let _ = out_tx.send(ser(&ServerMsg::Error {
                            message: format!("atrasou {} estados", n), fatal: false,
                        })).await;
                    }
                    Err(_) => break,
                }
            }

            msg = stream.next() => {
                let Some(Ok(Message::Text(txt))) = msg else { break };
                let Some(client_msg) = de::<ClientMsg>(&txt) else {
                    let _ = out_tx.send(ser(&ServerMsg::Error {
                        message: "mensagem inválida".into(), fatal: false,
                    })).await;
                    continue;
                };

                match client_msg {
                    ClientMsg::Join { token } => {
                        match room.join(&token).await {
                            Ok(GameState::JoinInfo { joined, .. }) => {
                                let _ = out_tx.send(ser(&ServerMsg::Joined {
                                    name: joined, room_id: room_id.clone(),
                                })).await;
                            }
                            Ok(_) => unreachable!("join só devolve JoinInfo"),
                            Err(e) => {
                                let _ = out_tx.send(ser(&ServerMsg::Error {
                                    message: err_text(&e), fatal: false,
                                })).await;
                            }
                        }
                    }

                    ClientMsg::Play { token, input } => {
                        match room.play(&token, &input).await {
                            Ok(st) => {
                                let _ = tx.send(st.clone());
                                let json = ser(&ServerMsg::State { state: st });
                                skip_echo = Some(json.clone());
                                let _ = out_tx.send(json).await;
                            }
                            Err(e) => {
                                let _ = out_tx.send(ser(&ServerMsg::Error {
                                    message: err_text(&e), fatal: false,
                                })).await;
                            }
                        }
                    }
                }
            }
        }
    }

    writer.abort();
}
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use site::{
    app::create_router, core::env::EnvSetting, repositories::{
        games::{GameHub, GameKind}, github::github::GitHub, httpc::httpc::HttpC,
    },
};

async fn wait_piece<S, E>(r: &mut S, piece: &str) -> String
    where
        S: futures::Stream<Item = Result<Message, E>> + Unpin,
        E: std::fmt::Debug, 
    {
        let needle = format!("| {} ", piece);
        loop {
            let msg = r.next().await
                .expect("stream fechou antes do estado esperado") 
                .expect("erro no websocket"); 
            let txt = msg.to_string();
            if txt.contains(&needle) {
                return txt;
            }
    }
}

async fn spawn_server(vs_cpu: bool) -> (String, String, String) {
    let env = std::sync::Arc::new(EnvSetting::new());
    let http = HttpC::new(&env);
    let gh = std::sync::Arc::new(GitHub::new(&env, &http));
    let games = std::sync::Arc::new(GameHub::new(3, 4));

    let created = GameKind::Connect4
        .create(&games, "alice".into(), 'X', vs_cpu).unwrap();

    let app = create_router(gh, env, games);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });

    (addr, created.room_id, created.host_token)
}

// POST /games/rooms/{room_id}/join
async fn claim(addr: &str, room_id: &str, body: serde_json::Value) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!("http://{}/games/rooms/{}/join", addr, room_id))
        .json(&body)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn pvp_broadcast_entrega_para_os_dois() {
    let (addr, room_id, t1) = spawn_server(false).await;

    let resp = claim(&addr, &room_id, serde_json::json!({"name": "bob", "fig": "&"})).await;
    assert_eq!(resp.status(), 200);
    let seat: serde_json::Value = resp.json().await.unwrap();
    let t2 = seat["token"].as_str().unwrap().to_string();

    let (s1, _) = tokio_tungstenite::connect_async(
        format!("ws://{}/games/ws/{}?token={}", addr, room_id, t1)).await.unwrap();
    let (s2, _) = tokio_tungstenite::connect_async(
        format!("ws://{}/games/ws/{}?token={}", addr, room_id, t2)).await.unwrap();
    let (mut w1, mut r1) = s1.split();
    let (mut w2, mut r2) = s2.split();

    let j1 = r1.next().await.unwrap().unwrap().to_string();
    let j2 = r2.next().await.unwrap().unwrap().to_string();
    assert!(j1.contains("\"joined\""), "host recebeu Joined: {j1}");
    assert!(j2.contains("bob"), "guest entrou com o nome do claim: {j2}");

    w1.send(Message::text(format!(
        r#"{{"action":"play","data":{{"token":"{}","input":"4"}}}}"#, t1))).await.unwrap();

    let st1 = wait_piece(&mut r1, "X").await;
    let st2 = wait_piece(&mut r2, "X").await;
    assert_eq!(st1, st2, "ambos veem o MESMO estado");

    w2.send(Message::text(format!(
        r#"{{"action":"play","data":{{"token":"{}","input":"1"}}}}"#, t2))).await.unwrap();

    let st2n = wait_piece(&mut r2, "&").await;
    let st1n = wait_piece(&mut r1, "&").await;
    assert_eq!(st1n, st2n, "estados idênticos após a 2ª jogada");
}

#[tokio::test]
async fn join_room_claim_valido_e_atrasado_fica_fora() {
    let (addr, room_id, _t1) = spawn_server(false).await;

    let resp = claim(&addr, &room_id, serde_json::json!({"name": "bob", "fig": "&"})).await;
    assert_eq!(resp.status(), 200);
    let seat: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(seat["joined"], "bob");
    assert_eq!(seat["room_id"], room_id);
    assert!(seat["token"].as_str().map_or(false, |t| !t.is_empty()), "token veio");

    let resp = claim(&addr, &room_id, serde_json::json!({})).await;
    assert_eq!(resp.status(), reqwest::StatusCode::CONFLICT);

    let token = seat["token"].as_str().unwrap().to_string();
    let (s, _) = tokio_tungstenite::connect_async(
        format!("ws://{}/games/ws/{}?token={}", addr, room_id, token)).await.unwrap();
    let (_w, mut r) = s.split();
    let first = r.next().await.unwrap().unwrap().to_string();
    assert!(first.contains("bob"), "WS aceitou o assento claimeado: {first}");
}

#[tokio::test]
async fn join_room_figura_em_uso_nao_ocupa_assento() {
    let (addr, room_id, _t1) = spawn_server(false).await;

    let resp = claim(&addr, &room_id, serde_json::json!({"name": "clonado", "fig": "X"})).await;
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);

    let resp = claim(&addr, &room_id, serde_json::json!({"name": "bob", "fig": "&"})).await;
    assert_eq!(resp.status(), 200);
    let seat: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(seat["joined"], "bob", "transação: nada foi escrito no claim recusado");
}

#[tokio::test]
async fn join_room_vs_cpu_nao_tem_assento() {
    let (addr, room_id, _t1) = spawn_server(true).await;

    let resp = claim(&addr, &room_id, serde_json::json!({"name": "intruso"})).await;
    assert_eq!(resp.status(), reqwest::StatusCode::CONFLICT);
}

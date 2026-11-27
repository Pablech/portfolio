use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use serde_json::json;

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
    let needle = format!("> {} <", piece);
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let msg = r.next().await
                .expect("stream fechou antes do estado esperado")
                .expect("erro no websocket");
            let txt = msg.to_string();
            if txt.contains(&needle) {
                return txt;
            }
        }
    })
    .await
    .expect("timeout aguardando peça no websocket")
}

async fn err_tuple(resp: reqwest::Response, code: u16) -> serde_json::Value {
    assert_eq!(resp.status(), reqwest::StatusCode::from_u16(code).unwrap(),
        "status HTTP deve refletir o código da tupla");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], code, "corpo deve trazer {code} em code: {body}");
    assert!(body["message"].as_str().is_some_and(|m| !m.is_empty()),
        "message não pode vir vazia — é ela que o terminal imprime: {body}");
    body
}

async fn wait_status<S, E>(r: &mut S, status: &str) -> (serde_json::Value, String)
where
    S: futures::Stream<Item = Result<Message, E>> + Unpin,
    E: std::fmt::Debug,
{
    loop {
        let txt = r.next().await
            .expect("stream fechou antes do esperado")
            .expect("erro no websocket")
            .to_string();
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
            if v["status"] == status { return (v, txt); }
        }
    }
}

async fn spawn_server(vs_cpu: bool) -> (String, String, String) {
    let env = std::sync::Arc::new(EnvSetting::new());
    let http = HttpC::new(&env);
    let gh = std::sync::Arc::new(GitHub::new(&env, &http));
    let games = std::sync::Arc::new(GameHub::new(3, 4, 2));

    let created = GameKind::Connect4
        .create(&games, Some("alice".to_string()), Some('X'), vs_cpu).unwrap();

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
    assert!(seat["token"].as_str().is_some_and(|t| !t.is_empty()), "token veio");

    let e = err_tuple(claim(&addr, &room_id, serde_json::json!({})).await, 403).await;
    assert!(e["message"].as_str().unwrap().contains("sala") || e["message"].as_str().unwrap().contains("cheia"),
        "mensagem humana útil: {e}");

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

    let e = err_tuple(claim(&addr, &room_id, json!({"name": "clonado", "fig": "X"})).await, 422).await;
    assert!(e["message"].as_str().unwrap().contains("figura"), "{e}");

    let resp = claim(&addr, &room_id, serde_json::json!({"name": "bob", "fig": "&"})).await;
    assert_eq!(resp.status(), 200);
    let seat: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(seat["joined"], "bob", "transação: nada foi escrito no claim recusado");
}

#[tokio::test]
async fn join_room_vs_cpu_nao_tem_assento() {
    let (addr, room_id, _t1) = spawn_server(true).await;

    let e = err_tuple(claim(&addr, &room_id, json!({"name": "intruso"})).await, 403).await;
    assert!(e["message"].as_str().unwrap().contains("CPU") || e["message"].as_str().unwrap().contains("cheia"), "{e}");
}

#[tokio::test]
async fn chat_chega_pros_dois_e_nao_vaza_token() {
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

    let (_, _) = wait_status(&mut r1, "joined").await;
    let (_, _) = wait_status(&mut r2, "joined").await;

    w1.send(Message::text(format!(
        r#"{{"action":"chat","data":{{"token":"{}","text":"  boa jogada!  "}}}}"#, t1))).await.unwrap();

    let (v1, c1) = wait_status(&mut r1, "chat").await;
    let (v2, c2) = wait_status(&mut r2, "chat").await;

    assert_eq!(v1, v2, "ambos recebem o mesmo chat");
    assert_eq!(v1["author"], "alice", "author é o nome, não o token");
    assert_eq!(v1["text"], "boa jogada!", "texto trimado");
    assert!(!c1.contains(&t1), "token de alice NÃO pode vazar: {c1}");
    assert!(!c2.contains(&t1), "token de alice NÃO pode vazar pro guest: {c2}");

    w2.send(Message::text(r#"{"action":"chat","data":{"token":"golpista","text":"oi?"}}"#.to_string())).await.unwrap();
    let (ev, _) = wait_status(&mut r2, "error").await;
    assert_eq!(ev["code"], 403, "token inválido é ErrorJoin -> 403");
    assert!(ev["message"].as_str().unwrap_or("").contains("token"));

    w2.send(Message::text(format!(
        r#"{{"action":"chat","data":{{"token":"{}","text":"beleza"}}}}"#, t2))).await.unwrap();
    let (v1b, _) = wait_status(&mut r1, "chat").await;
    let (v2b, _) = wait_status(&mut r2, "chat").await;
    assert_eq!(v1b["author"], "bob", "próximo chat é do bob — o inválido não entrou no canal");
    assert_eq!(v1b, v2b);
}

#[tokio::test]
async fn ws_sem_token_ou_invalido_e_rejeitado_com_401() {
    let (addr, room_id, _t1) = spawn_server(false).await;

    let err_no_token = tokio_tungstenite::connect_async(
        format!("ws://{}/games/ws/{}", addr, room_id)
    ).await.unwrap_err();

    match err_no_token {
        tokio_tungstenite::tungstenite::Error::Http(resp) => {
            assert_eq!(resp.status(), 401);
        }
        other => panic!("esperado HTTP 401 sem token, veio: {:?}", other),
    }

    let err_bad_token = tokio_tungstenite::connect_async(
        format!("ws://{}/games/ws/{}?token=invalido", addr, room_id)
    ).await.unwrap_err();

    match err_bad_token {
        tokio_tungstenite::tungstenite::Error::Http(resp) => {
            assert_eq!(resp.status(), 401);
        }
        other => panic!("esperado HTTP 401 com token inválido, veio: {:?}", other),
    }
}

#[tokio::test]
async fn req_id_vai_apenas_para_quem_executou_a_jogada() {
    let (addr, room_id, t1) = spawn_server(false).await;

    let resp = claim(&addr, &room_id, json!({"name": "bob", "fig": "&"})).await;
    assert_eq!(resp.status(), 200);
    let t2 = resp.json::<serde_json::Value>().await.unwrap()["token"]
        .as_str().unwrap().to_string();

    let (s1, _) = tokio_tungstenite::connect_async(
        format!("ws://{}/games/ws/{}?token={}", addr, room_id, t1)
    ).await.unwrap();
    let (s2, _) = tokio_tungstenite::connect_async(
        format!("ws://{}/games/ws/{}?token={}", addr, room_id, t2)
    ).await.unwrap();

    let (mut w1, mut r1) = s1.split();
    let (_w2, mut r2) = s2.split();

    // Consome mensagens iniciais (Joined / snapshot inicial)
    let _ = r1.next().await;
    let _ = r2.next().await;

    w1.send(Message::text(format!(
        r#"{{"action":"play","data":{{"token":"{}","input":"4","req_id":"req-99"}}}}"#,
        t1
    ))).await.unwrap();

    let st1 = wait_piece(&mut r1, "X").await;
    let st2 = wait_piece(&mut r2, "X").await;

    let v1: serde_json::Value = serde_json::from_str(&st1).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&st2).unwrap();

    assert_eq!(v1["req_id"], "req-99");
    assert!(v2["req_id"].is_null(), "outro assento não deve receber req_id do ator: {v2}");
}
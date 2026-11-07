
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use site::{
    app::create_router,
    repositories::{
        games::{
            GameHub, GameKind,
        },
        github::github::GitHub,
        httpc::httpc::HttpC,
    },
    core::env::EnvSetting,
};

async fn spawn_server() -> (String, String, String) {
    let env = std::sync::Arc::new(EnvSetting::new());
    let http = HttpC::new(&env);
    let gh = std::sync::Arc::new(GitHub::new(&env, &http));
    let games = std::sync::Arc::new(GameHub::new());
    let created = GameKind::Connect4
        .create(&games, "alice".into(), 'X',false).unwrap();

    let app = create_router(gh, env, games);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let url = format!("ws://{}/games/ws/{}", addr, created.room_id);
    (url, created.host_token, created.guest_token.unwrap())
}

#[tokio::test]
async fn pvp_broadcast_entrega_para_os_dois() {
    let (url, t1, t2) = spawn_server().await;

    let (s1, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    let (s2, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    let (mut w1, mut r1) = s1.split();
    let (mut w2, mut r2) = s2.split();

    w1.send(Message::text(r#"{"action":"join","data":{"token":"TOKEN1"}}"#.replace("TOKEN1", &t1))).await.unwrap();
    w2.send(Message::text(r#"{"action":"join","data":{"token":"TOKEN2"}}"#.replace("TOKEN2", &t2))).await.unwrap();
    let j1 = r1.next().await.unwrap().unwrap(); 
    let j2 = r2.next().await.unwrap().unwrap();
    assert!(j1.to_string().contains("\"joined\""));
    assert!(j2.to_string().contains("alice") || j2.to_string().contains("p2"));

    w1.send(Message::text(format!(
        r#"{{"action":"play","data":{{"token":"{}","input":"4"}}}}"#, t1))).await.unwrap();

    let st1 = r1.next().await.unwrap().unwrap().to_string();
    let st2 = r2.next().await.unwrap().unwrap().to_string();

    assert!(st1.contains("X"), "autor recebeu a própria jogada");
    assert!(st2.contains("X"), "convidado recebeu via broadcast sem jogar");
    assert_eq!(st1, st2, "ambos devem ver o MESMO estado");

    w2.send(Message::text(format!(
        r#"{{"action":"play","data":{{"token":"{}","input":"1"}}}}"#, t2))).await.unwrap();
    let s2n = r2.next().await.unwrap().unwrap().to_string(); 
    let s1n = r1.next().await.unwrap().unwrap().to_string(); 
    assert!(s2n.contains("&"), "peça do bob no board");
    assert_eq!(s1n, s2n);
}

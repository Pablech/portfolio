use std::sync::Arc;
use axum::{extract::State, http::StatusCode, Json};

use crate::{handlers::structs::{CreateRoomReq, CreateRoomResp}, repositories::games::{
    CreatedRoom, GameHub, GameKind,
}};

// route = POST /api/games/rooms
pub async fn create_room(State(hub): State<Arc<GameHub>>,
    Json(req): Json<CreateRoomReq>, ) -> Result<Json<CreateRoomResp>, (StatusCode, String)> {

    let kind = GameKind::from_str(&req.game)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let CreatedRoom { room_id, host_token, guest_token } = kind
        .create(
            &hub,
            req.name.unwrap_or_else(|| "p1".into()),
            req.fig.unwrap_or('X'),
            req.vs_cpu.unwrap_or(true),
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CreateRoomResp {
        room_id,
        token: host_token,
        guest_token,
    }))
}

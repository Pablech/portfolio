use std::sync::Arc;
use axum::{Json, extract::{Path, State}, http::StatusCode};

use crate::{handlers::structs::{CreateRoomReq, CreateRoomResp, JoinRoomReq, JoinRoomResp}, 
repositories::games::{
    CreatedRoom, GameError, GameHub, GameKind,
}};

// route = POST /games/rooms
pub async fn create_room(State(hub): State<Arc<GameHub>>,
    Json(req): Json<CreateRoomReq>, ) -> Result<Json<CreateRoomResp>, (StatusCode, String)> {

    let kind = GameKind::from_str(&req.game)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let CreatedRoom { room_id, host_token } = kind
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
    }))
}

// route = POST /games/rooms/:room_id/join
pub async fn join_room(State(hub): State<Arc<GameHub>>,
    Path(room_id): Path<String>, Json(req): Json<JoinRoomReq>,
    ) -> Result<Json<JoinRoomResp>, (StatusCode, String)> {

    let room = hub.get_room(&room_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "sala inexistente".into()))?;

    let seat = room.claim(req.name, req.fig).await
        .map_err(|e| {
            let status = match &e {
                GameError::ErrorJoin(_) => StatusCode::CONFLICT,
                GameError::ErrorValidation(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, format!("{:.?}", e))
        })?;
    
    Ok(Json(JoinRoomResp { token: seat.token, joined: seat.joined, room_id: seat.room_id }))
}
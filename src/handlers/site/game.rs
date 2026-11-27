use std::{str::FromStr, sync::Arc};
use axum::{Json, extract::{Path, State}, http::StatusCode};

use crate::{handlers::structs::{ApiError, CreateRoomReq, CreateRoomResp, JoinRoomReq, JoinRoomResp}, repositories::games::{
    CreatedRoom, GameHub, GameKind,
}};

// route = POST /games/rooms
pub async fn create_room(State(hub): State<Arc<GameHub>>,
    Json(req): Json<CreateRoomReq>, ) -> Result<Json<CreateRoomResp>, ApiError> {

    let kind = GameKind::from_str(&req.game).map_err(ApiError::from)?;

    let CreatedRoom { room_id, host_token } = kind
        .create(&hub, req.name, req.fig, req.vs_cpu.unwrap_or(true))?; 

    Ok(Json(CreateRoomResp { room_id, token: host_token }))
}

// route = POST /games/rooms/:room_id/join
pub async fn join_room(State(hub): State<Arc<GameHub>>,
    Path(room_id): Path<String>, Json(req): Json<JoinRoomReq>,
    ) -> Result<Json<JoinRoomResp>, ApiError> {

    let room = hub.get_room(&room_id)
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, format!("Sala {} inexistente", room_id)))?;

    let seat = room.claim(req.name, req.fig).await?; 
    
    Ok(Json(JoinRoomResp { token: seat.token, joined: seat.joined, room_id: seat.room_id }))
}
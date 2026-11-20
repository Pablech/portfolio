use axum::http::StatusCode;

use crate::repositories::games::{GameError, WsError};

pub enum Frame {
    Text(String), 
    Skip,         
    Close,        
}

#[derive(PartialEq)]
pub enum Flow {
    Continue,
    Close,
}


impl GameError {
    pub fn code(&self) -> u16 {
        match self {
            GameError::ErrorJoin(_) => 403,
            GameError::ErrorMove(_) => 409,
            GameError::ErrorValidation(_) => 422,
            GameError::ErrorNotFound(_) => 404,
            GameError::ErrorProtocol(_) => 400,
            
        }
    }

    pub fn erro(&self) -> &str {
        match self {
            GameError::ErrorJoin(m) => m,
            GameError::ErrorMove(m) => m,
            GameError::ErrorValidation(m) => m,
            GameError::ErrorNotFound(m) => m,
            GameError::ErrorProtocol(m) => m,
        }
    }

    pub fn as_tuple(&self) -> (u16, &str) {
        (self.code(), self.erro())
    }
}

impl From<GameError> for WsError {
    fn from(e: GameError) -> Self {
        let (code, err) = e.as_tuple();
        WsError(
            StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), 
            err.to_string(),
        )
    }
}
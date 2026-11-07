use std::sync::Arc;
use axum::extract::FromRef;

use crate::{core::env::EnvSetting, repositories::{games::GameHub, github::github::GitHub}};

#[derive(Clone)]
pub struct AppState {
    pub gh: Arc<GitHub>,
    pub env: Arc<EnvSetting>,
    pub games: Arc<GameHub>,
}

impl FromRef<AppState> for Arc<GitHub> {
    fn from_ref(state: &AppState) -> Arc<GitHub> {
        state.gh.clone()
    }
}

impl FromRef<AppState> for Arc<EnvSetting> {
    fn from_ref(state: &AppState) -> Arc<EnvSetting> {
        state.env.clone()
    }
}

impl FromRef<AppState> for Arc<GameHub> {
    fn from_ref(state: &AppState) -> Arc<GameHub> {
        state.games.clone()
    }
}
use super::{
    config::{Light, Mode},
    util::AppError,
    AppState,
};
use crate::CONFIG;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use axum_auth::AuthBearer;
use std::sync::Arc;

pub async fn set_mode(
    Path(mode): Path<Mode>,
    AuthBearer(token): AuthBearer,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, AppError> {
    if token != CONFIG.key {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    state.config.lock().await.mode = mode;

    Ok(StatusCode::OK)
}

pub async fn set_light(
    Path(light): Path<Light>,
    AuthBearer(token): AuthBearer,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, AppError> {
    if token != CONFIG.key {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    state.config.lock().await.custom = light;

    Ok(StatusCode::OK)
}

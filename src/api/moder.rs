use super::{
    state::{Light, Mode},
    util::AppError,
    AppState,
};
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
    if token != state.key {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    state.state.lock().await.mode = mode;

    Ok(StatusCode::OK)
}

pub async fn set_light(
    Path(light): Path<Light>,
    AuthBearer(token): AuthBearer,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, AppError> {
    if token != state.key {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    state.state.lock().await.custom = light;

    Ok(StatusCode::OK)
}

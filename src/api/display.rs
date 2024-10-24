use super::{state::Mode, util::AppError, AppState};
use axum::extract::State;
use std::sync::Arc;

pub async fn get_live(State(state): State<Arc<AppState>>) -> Result<String, AppError> {
    let config = state.state.lock().await;
    let mode = config.mode;
    let custom = config.custom;
    drop(config);
    let light = match mode {
        Mode::Static => custom,
        Mode::Bakalari => state.bakalari.get_state().await?.light(),
        Mode::Random => rand::random(),
    };
    Ok(format!("{}", light.to_val() | 0b1000))
}

use super::{state::Mode, util::AppError, AppState};
use axum::extract::State;
use std::sync::Arc;

pub async fn get_live(State(state): State<Arc<AppState>>) -> Result<String, AppError> {
    let config = state.state.lock().await;
    let static_val = config.custom.to_val();
    let mode = config.mode;
    drop(config);
    let light = match mode {
        Mode::Static => static_val,
        Mode::Bakalari => state.bakalari.get_state().await?.light().to_val(),
    };
    Ok(format!("{}", light | 0b1000))
}

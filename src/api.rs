use crate::cli::Config;
use axum::{
    routing::{get, post},
    Router,
};
use bakalari::BakaWrapper;
use display::get_live;
use moder::{set_light, set_mode};
use rezvrh_scraper::Bakalari;
use state::State;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use util::shutdown_signal;

mod bakalari;
mod display;
mod moder;
mod state;
mod util;

struct AppState {
    bakalari: BakaWrapper,
    state: Mutex<State>,
    key: String,
}

pub async fn api(config: Config) -> anyhow::Result<()> {
    let bakalari = if let Some(creds) = config.bakalari.auth.clone() {
        Bakalari::from_creds(creds, config.bakalari.url.clone()).await?
    } else {
        Bakalari::no_auth(config.bakalari.url.clone()).await?
    };

    let bakalari = bakalari::BakaWrapper::new(bakalari, &config.bakalari.room, config.time).await?;
    let state = Mutex::new(State::default());

    let socket = config.socket;

    let state = Arc::new(AppState {
        bakalari,
        state,
        key: config.key,
    });

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/config", get(get_live))
        .route("/mode/:mode", post(set_mode))
        .route("/light/:light", post(set_light))
        .with_state(state)
        .layer(CorsLayer::very_permissive());

    let listener = tokio::net::TcpListener::bind(socket).await?;
    tracing::info!("Starting...");
    Ok(axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?)
}

use crate::CONFIG;
use axum::{
    routing::{get, post},
    Router,
};
use bakalari::BakaWrapper;
use config::Config;
use display::get_live;
use moder::{set_light, set_mode};
use rezvrh_scraper::Bakalari;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use util::shutdown_signal;

mod bakalari;
mod config;
mod display;
mod moder;
mod util;

struct AppState {
    bakalari: BakaWrapper,
    config: Mutex<Config>,
}

pub async fn api() -> anyhow::Result<()> {
    let bakalari = if let Some(creds) = CONFIG.bakalari.auth.clone() {
        Bakalari::from_creds(creds, CONFIG.bakalari.url.clone()).await?
    } else {
        Bakalari::no_auth(CONFIG.bakalari.url.clone()).await?
    };

    let bakalari = bakalari::BakaWrapper::new(bakalari).await?;
    let config = Mutex::new(Config::default());

    let state = Arc::new(AppState { bakalari, config });

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/config", get(get_live))
        .route("/mode/:mode", post(set_mode))
        .route("/light/:light", post(set_light))
        .with_state(state)
        .layer(CorsLayer::very_permissive());

    let listener = tokio::net::TcpListener::bind(CONFIG.socket).await?;
    tracing::info!("Starting...");
    Ok(axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?)
}

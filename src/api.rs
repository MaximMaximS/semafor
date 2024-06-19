use crate::CONFIG;
use anyhow::Context;
use axum::{routing::get, Router};
use bakalari::get_light;
use rezvrh_scraper::Bakalari;
use std::sync::Arc;
use util::shutdown_signal;

mod bakalari;
mod util;

pub async fn api() -> anyhow::Result<()> {
    let bakalari = if let Some(creds) = CONFIG.bakalari.auth.clone() {
        Bakalari::from_creds(creds, CONFIG.bakalari.url.clone()).await?
    } else {
        Bakalari::no_auth(CONFIG.bakalari.url.clone()).await?
    };

    let bakalari = Arc::new(bakalari::BakaWrapper::new(bakalari).context("Invalid room")?);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/config", get(get_light))
        .with_state(bakalari);

    let listener = tokio::net::TcpListener::bind(CONFIG.socket).await?;
    tracing::info!("Starting...");
    Ok(axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?)
}

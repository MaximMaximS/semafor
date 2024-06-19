use crate::CONFIG;
use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use rezvrh_scraper::Bakalari;
use std::sync::Arc;
use tokio::signal;

mod bakalari;

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
async fn get_light(State(baka): State<Arc<bakalari::BakaWrapper>>) -> Result<String, AppError> {
    let state = baka.get_state().await?;
    Ok(format!("{}", state.light() | 0b1000))
}

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
    Ok(axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?)
}

#[allow(clippy::redundant_pub_crate)]
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("Shutting down...");
}

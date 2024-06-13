use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use bakalari::BakaWrapper;
use rezvrh_scraper::Bakalari;
use std::{env::var, sync::Arc};

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
async fn get_light(State(baka): State<Arc<BakaWrapper>>) -> Result<String, AppError> {
    let state = baka.get_state().await?;
    Ok(format!("1{:0>3}", state.light()))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    // env
    let address = var("ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = var("PORT").unwrap_or_else(|_| "3000".to_string());

    // Baka
    let room = var("BAKA_ROOM").context("BAKA_ROOM not set")?;
    let baka_url = var("BAKA_URL").context("BAKA_URL not set")?.parse()?;
    let baka_auth = var("BAKA_AUTH").ok();

    let lights_on = var("LIGHTS_ON")
        .context("LIGHTS_ON not set")?
        .parse::<u32>()
        .context("Invalid LIGHTS_ON")?;
    let lights_off = var("LIGHTS_OFF")
        .context("LIGHTS_OFF not set")?
        .parse::<u32>()
        .context("Invalid LIGHTS_OFF")?;
    let before_break = var("BEFORE_BREAK")
        .context("BEFORE_BREAK not set")?
        .parse::<u32>()
        .context("Invalid BEFORE_BREAK")?;
    let before_lesson = var("BEFORE_LESSON")
        .context("BEFORE_LESSON not set")?
        .parse::<u32>()
        .context("Invalid BEFORE_LESSON")?;

    let creds = if let Some(auth) = baka_auth {
        let (username, password) = auth.split_once(':').context("Invalid BAKA_AUTH")?;
        Some((username.to_owned(), password.to_owned()))
    } else {
        None
    };

    let bakalari = if let Some(creds) = creds {
        Bakalari::from_creds(creds, baka_url).await?
    } else {
        Bakalari::no_auth(baka_url).await?
    };

    let options = bakalari::Options {
        lights_on,
        lights_off,
        before_break,
        before_lesson,
    };
    let bakalari =
        Arc::new(bakalari::BakaWrapper::new(bakalari, &room, options).context("Invalid room")?);

    let app = Router::new()
        .route("/config", get(get_light))
        .with_state(bakalari);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("{address}:{port}")).await?;
    Ok(axum::serve(listener, app).await?)
}

use anyhow::Context;
use axum::{
    routing::get,
    Router,
};
use rezvrh_scraper::Bakalari;
use std::env::var;

mod bakalari;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    // env
    let address = var("ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = var("PORT").unwrap_or_else(|_| "3000".to_string());

    // Baka
    let baka_url = var("BAKA_URL").context("BAKA_URL not set")?.parse()?;
    let baka_auth = var("BAKA_AUTH").ok();
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

    let bakalari = bakalari::BakaWrapper::new(bakalari);

    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("{address}:{port}")).await?;
    Ok(axum::serve(listener, app).await?)
}
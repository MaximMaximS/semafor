use tracing::Level;
use tracing_subscriber::{fmt::format::FmtSpan, FmtSubscriber};

mod api;
pub(crate) mod bakalari;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    api::api().await?;

    Ok(())
}

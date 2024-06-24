mod api;
mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = cli::init()?;

    api::api(config).await?;

    Ok(())
}

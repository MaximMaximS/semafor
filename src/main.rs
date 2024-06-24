mod api;
mod args;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = args::init()?;

    api::api(config).await?;

    Ok(())
}

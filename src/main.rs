use std::{ops::Deref, sync::OnceLock};

use cli::Config;

mod api;
mod cli;

#[derive(Debug)]
struct ConfigWrapper(OnceLock<Config>);

static CONFIG: ConfigWrapper = ConfigWrapper(OnceLock::new());

impl Deref for ConfigWrapper {
    type Target = Config;
    fn deref(&self) -> &Self::Target {
        self.0.get().unwrap()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = cli::init()?;
    CONFIG.0.set(config).unwrap();

    api::api().await?;

    Ok(())
}

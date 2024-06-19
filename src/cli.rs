use anyhow::Context;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Duration;
use clap::Parser;
use std::net::{IpAddr, SocketAddr};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use url::Url;

/// Semafor control server
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Address to bind to
    #[clap(long, env, default_value = "0.0.0.0")]
    address: IpAddr,
    /// Port to bind to
    #[clap(short, long, env, default_value = "3000", value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,

    /// Increase logging verbosity
    #[clap(short, long)]
    verbose: bool,

    /// Room to control
    #[clap(long, env)]
    bakalari_room: String,

    /// Bakalari URL
    #[clap(long, env)]
    bakalari_url: Url,

    /// Bakalari auth
    #[clap(long, env)]
    bakalari_auth: Option<String>,

    /// Lights on time
    #[clap(long, env, default_value = "1800")]
    lights_on: u32,

    /// Lights off time
    #[clap(long, env, default_value = "600")]
    lights_off: u32,

    /// Time before break
    #[clap(long, env, default_value = "60")]
    before_break: u32,

    /// Time before lesson
    #[clap(long, env, default_value = "60")]
    before_lesson: u32,

    /// Admin key
    #[clap(long)]
    key: String,
}

#[derive(Debug)]
pub struct TimeOptions {
    /// Seconds
    pub lights_on: Duration,
    /// Seconds
    pub lights_off: Duration,
    /// Seconds
    pub before_break: Duration,
    /// Seconds
    pub before_lesson: Duration,
}

#[derive(Debug)]
pub struct BakalariOptions {
    pub url: Url,
    pub auth: Option<(String, String)>,
    pub room: String,
}

#[derive(Debug)]
pub struct Config {
    pub socket: SocketAddr,
    pub bakalari: BakalariOptions,
    pub time: TimeOptions,
    pub key: String,
}

impl TryFrom<Args> for Config {
    type Error = anyhow::Error;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let auth = args
            .bakalari_auth
            .map(|b| STANDARD.decode(b))
            .transpose()?
            .map(String::from_utf8)
            .transpose()?
            .map(|s| {
                s.split_once(':')
                    .context("auth split failed")
                    .map(|p| (p.0.to_owned(), p.1.to_owned()))
            })
            .transpose()?;

        Ok(Self {
            socket: SocketAddr::new(args.address, args.port),
            bakalari: BakalariOptions {
                url: args.bakalari_url,
                auth,
                room: args.bakalari_room,
            },
            time: TimeOptions {
                lights_on: Duration::seconds(i64::from(args.lights_on)),
                lights_off: Duration::seconds(i64::from(args.lights_off)),
                before_break: Duration::seconds(i64::from(args.before_break)),
                before_lesson: Duration::seconds(i64::from(args.before_lesson)),
            },
            key: args.key,
        })
    }
}

pub fn init() -> anyhow::Result<Config> {
    let args = Args::parse();
    let level = match (args.verbose, cfg!(debug_assertions)) {
        (false, false) => Level::INFO,
        (false, true) | (true, false) => Level::DEBUG,
        (true, true) => Level::TRACE,
    };
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let config = Config::try_from(args)?;
    Ok(config)
}

use std::{fs::File, path::PathBuf};

use anyhow::Context;
use apply_inovelli_defaults::config;
use clap::Parser;
use std::num::NonZeroU32;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use url::Url;

#[derive(Debug, PartialEq, Eq, Parser)]
struct Args {
    /// The address of the websocket endpoint for your zigbee2mqtt installation
    zigbee2mqtt_url: Url,

    /// The configuration file to use
    config_file: PathBuf,

    /// Whether to really apply the settings.
    #[clap(short, long, default_value = "false")]
    real: bool,

    /// The maximum number of updates to send through zigbee2mqtt; defaults to unlimited
    #[clap(long)]
    messages_per_second: Option<NonZeroU32>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    tracing::debug!(cmdline = ?args, "Starting");
    let config_file = File::open(&args.config_file)?;
    let config: Vec<config::ConfigClause> = serde_yaml::from_reader(config_file)
        .with_context(|| format!("Parsing config file {:?}", &args.config_file))?;

    let mut conn = apply_inovelli_defaults::Connection::connect(&args.zigbee2mqtt_url, args.real)
        .await
        .context("Can't connect to zigbee2mqtt websocket endpoint")?;
    tracing::info!(addr = %args.zigbee2mqtt_url, conn=?conn, "Connected");

    let lim = args
        .messages_per_second
        .map(|n| governor::RateLimiter::direct(governor::Quota::per_second(n)));
    conn.update_loop(config, lim).await?;
    Ok(())
}

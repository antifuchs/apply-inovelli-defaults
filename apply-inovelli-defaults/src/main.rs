use std::{fs::File, path::PathBuf};

use anyhow::Context;
use apply_inovelli_defaults::config;
use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use url::Url;

#[derive(Debug, PartialEq, Eq, Parser)]
struct Args {
    /// The address of the websocket endpoint for your zigbee2mqtt installation
    zigbee2mqtt_url: Url,
    config_file: PathBuf,
    #[clap(short, long, default_value = "false")]
    real: bool,
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
    conn.update_loop(config).await?;
    Ok(())
}

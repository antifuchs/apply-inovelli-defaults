use anyhow::Context;
use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use url::Url;

#[derive(Debug, PartialEq, Eq, Parser)]
struct Args {
    /// The address of the websocket endpoint for your zigbee2mqtt installation
    zigbee2mqtt_url: Url,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    tracing::debug!(cmdline = ?args, "Starting");

    let conn = apply_inovelli_defaults::Connection::connect(&args.zigbee2mqtt_url)
        .await
        .context("Can't connect to zigbee2mqtt websocket endpoint")?;
    tracing::info!(addr = %args.zigbee2mqtt_url, conn=?conn, "Connected");
    Ok(())
}

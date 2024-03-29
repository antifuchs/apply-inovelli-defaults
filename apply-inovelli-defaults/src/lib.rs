use anyhow::Context;
use config::ConfigClause;
use core::fmt;
use std::collections::{HashMap, HashSet};

use futures_util::sink::SinkExt;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;

pub mod config;
mod init_messages;
mod messages;

/// A zigbee2mqtt initialization websocket message regarding the bridge.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "topic")]
enum Z2mInitMessage {
    #[serde(rename = "bridge/state")]
    BridgeState { payload: init_messages::BridgeState },
    #[serde(rename = "bridge/info")]
    BridgeInfo { payload: init_messages::BridgeInfo },
    #[serde(rename = "bridge/devices")]
    Devices {
        payload: Vec<init_messages::BridgeDevice>,
    },
    #[serde(rename = "bridge/groups")]
    Groups {},
    #[serde(rename = "bridge/extensions")]
    Extensions {},
}

/// A zigbee2mqtt message that is part of the normal site communication protocol.
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Z2mMessage {
    /// A log message from zigbee2mqtt.
    Log {
        payload: messages::Log,
        topic: String,
    },
    /// A "null" message that seems to delineate some message group boundary. Unused.
    Null {
        payload: Option<String>,
        topic: String,
    },
    /// An "update" message returning the current values for a device in the network.
    Update {
        payload: HashMap<String, serde_json::Value>,
        topic: String,
    },
}

/// A zigbee2mqtt websocket message that we send to the endpoint.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Z2mUpdate {
    topic: String,
    payload: HashMap<String, serde_json::Value>,
}

impl TryInto<Vec<tokio_tungstenite::tungstenite::Message>> for Z2mUpdate {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<Vec<tokio_tungstenite::tungstenite::Message>, Self::Error> {
        let topic = self.topic;
        Ok(self
            .payload
            .into_iter()
            .map(|(k, v)| {
                let update = Z2mUpdate {
                    topic: topic.clone(),
                    payload: HashMap::from([(k, v)]),
                };
                tokio_tungstenite::tungstenite::Message::Text(
                    serde_json::to_string(&update).expect("Could not convert to JSON"),
                )
            })
            .collect())
    }
}

#[allow(dead_code)]
pub struct Connection {
    url: Url,
    write: SplitSink<
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        tokio_tungstenite::tungstenite::Message,
    >,
    read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    pub version: String,
    pub devices: HashSet<String>,
    real: bool,
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}Connection @{} v:{} devs:{:?}",
            if self.real { "Real " } else { "" },
            self.url,
            self.version,
            self.devices
        )
    }
}

async fn read_message<'de, T: serde::de::DeserializeOwned + fmt::Debug>(
    read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> anyhow::Result<T> {
    let Some(next) = read.next().await else {
        anyhow::bail!("No next message - counterparty hung up?");
    };
    let message = next.context("Error receiving next message")?;
    let data = message.into_data();
    let message =
        serde_json::from_reader(std::io::Cursor::new(data.clone())).with_context(move || {
            format!(
                "Could not parse message {:?}",
                String::from_utf8_lossy(&data)
            )
        })?;
    tracing::trace!(msg = ?message, "Read message data");
    Ok(message)
}

impl Connection {
    /// Connects to a zigbee2mqtt websocket stream.
    pub async fn connect(address: &Url, real: bool) -> anyhow::Result<Self> {
        let (ws_stream, _) = connect_async(address).await?;
        let (write, read) = ws_stream.split();
        Self::populate(read, write, address, real).await
    }

    async fn populate(
        read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        write: SplitSink<
            WebSocketStream<MaybeTlsStream<TcpStream>>,
            tokio_tungstenite::tungstenite::Message,
        >,
        url: &Url,
        real: bool,
    ) -> anyhow::Result<Self> {
        let mut read = read;
        let Z2mInitMessage::BridgeState { .. } = read_message(&mut read).await? else {
            anyhow::bail!("Could not read init bridge state");
        };
        let Z2mInitMessage::BridgeInfo {
            payload: init_messages::BridgeInfo { version },
        } = read_message(&mut read).await?
        else {
            anyhow::bail!("Could not read init bridge state");
        };
        let Z2mInitMessage::Devices { payload: devices } = read_message(&mut read).await? else {
            anyhow::bail!("Could not read the bridge's devices");
        };
        let Z2mInitMessage::Groups {} = read_message(&mut read).await? else {
            anyhow::bail!("Could not read the bridge's groups");
        };
        let Z2mInitMessage::Extensions {} = read_message(&mut read).await? else {
            anyhow::bail!("Could not read the bridge's extensions");
        };
        Ok(Self {
            read,
            write,
            url: url.clone(),
            version,
            devices: devices.into_iter().map(|dev| dev.topic_name).collect(),
            real,
        })
    }

    async fn send_update(
        &mut self,
        update: Z2mUpdate,
        rate_limiter: &Option<
            governor::RateLimiter<
                governor::state::NotKeyed,
                governor::state::InMemoryState,
                governor::clock::DefaultClock,
            >,
        >,
    ) -> anyhow::Result<()> {
        if !self.real {
            tracing::info!(?update, "Would send");
            return Ok(());
        }
        let messages: Vec<tokio_tungstenite::tungstenite::Message> = update.try_into()?;
        for message in messages {
            if let Some(limiter) = rate_limiter.as_ref() {
                limiter.until_ready().await;
            }
            tracing::debug!(?message, "sending update");
            self.write.send(message).await?;
        }
        Ok(())
    }

    pub async fn update_loop(
        &mut self,
        config: Vec<ConfigClause>,
        rate_limiter: Option<
            governor::RateLimiter<
                governor::state::NotKeyed,
                governor::state::InMemoryState,
                governor::clock::DefaultClock,
            >,
        >,
    ) -> anyhow::Result<Never> {
        let mut done = HashSet::new();
        loop {
            match read_message(&mut self.read)
                .await
                .context("Reading message in main loop")?
            {
                Z2mMessage::Update { topic, payload } => {
                    tracing::trace!(%topic, ?payload, "device update");
                    if done.contains(&topic) {
                        continue;
                    }
                    if let Some((rule_name, payload)) =
                        config.iter().find_map(|clause| clause.update_for(&payload))
                    {
                        tracing::info!(?topic, ?rule_name, "matched");
                        done.insert(topic.to_string());
                        let topic = format!("{topic}/set");

                        self.send_update(Z2mUpdate { topic, payload }, &rate_limiter)
                            .await?;
                    }
                }
                Z2mMessage::Log { topic, payload } => {
                    tracing::trace!(%topic, %payload.level, %payload.message);
                }
                msg => tracing::trace!(?msg, "received message"),
            }
        }
    }
}

pub enum Never {}

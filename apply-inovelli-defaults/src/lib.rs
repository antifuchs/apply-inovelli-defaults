use anyhow::Context;
use core::fmt;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;

mod init_messages;

/// A zigbee2mqtt initialization websocket message.
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
}

pub struct Connection {
    url: Url,
    write: SplitSink<
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        tokio_tungstenite::tungstenite::Message,
    >,
    read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    pub version: String,
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Connection @{} v:{}", self.url, self.version)
    }
}

impl Connection {
    /// Connects to a zigbee2mqtt websocket stream.
    pub async fn connect(address: &Url) -> anyhow::Result<Self> {
        let (ws_stream, _) = connect_async(address).await?;
        // check the bridge state:
        let (write, read) = ws_stream.split();

        Ok(Self::populate(read, write, address).await?)
    }

    async fn read_init_message(
        read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) -> anyhow::Result<Z2mInitMessage> {
        let Some(Ok(message)) = read.next().await else {
            anyhow::bail!("Ugh");
        };
        let data = message.into_data();
        let message = serde_json::from_reader(std::io::Cursor::new(data.clone())).with_context(
            move || {
                format!(
                    "Could not parse message {:?}",
                    String::from_utf8_lossy(&data)
                )
            },
        )?;
        tracing::debug!(msg = ?message, "Read data");
        Ok(message)
    }

    async fn populate(
        read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        write: SplitSink<
            WebSocketStream<MaybeTlsStream<TcpStream>>,
            tokio_tungstenite::tungstenite::Message,
        >,
        url: &Url,
    ) -> anyhow::Result<Self> {
        let mut read = read;
        let Z2mInitMessage::BridgeState{..} = Self::read_init_message(&mut read).await? else {
            anyhow::bail!("Could not read init bridge state");
        };
        let Z2mInitMessage::BridgeInfo{payload: init_messages::BridgeInfo{version}} = Self::read_init_message(&mut read).await? else {
            anyhow::bail!("Could not read init bridge state");
        };
        let Z2mInitMessage::Devices{payload: _devices} = Self::read_init_message(&mut read).await? else {
            anyhow::bail!("Could not read the bridge's devices");
        };
        Ok(Self {
            read,
            write,
            url: url.clone(),
            version,
        })
    }
}

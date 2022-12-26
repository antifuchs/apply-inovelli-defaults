use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct BridgeState {
    pub state: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct BridgeInfo {
    pub version: String,
    // TODO: several other fields that would be useful
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct BridgeDevice {
    #[serde(rename = "friendly_name")]
    pub topic_name: String,
    pub ieee_address: String,
    supported: bool,
    // TODO: even more other fields that would be useful
}

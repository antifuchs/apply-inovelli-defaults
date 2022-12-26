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
    pub friendly_name: String,
    pub ieee_address: String,
    supported: bool,
    // TODO: even more other fields that would be useful
}

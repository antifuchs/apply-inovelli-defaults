use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct State {
    pub state: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct Log {
    pub level: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct RefreshUpdate {
    pub state: String,
}

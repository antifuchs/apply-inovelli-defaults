use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

/// A configuration structure for this tool.
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigClause {
    /// Name of the configuration clause. Logged if it matches.
    name: Option<String>,

    /// Apply the configured `values` only if all the conditions here
    /// match (i.e. the values of the device are the ones in the
    /// zigbee2mqtt websocket message).
    condition: BTreeMap<String, serde_json::Value>,

    /// The values to send in a zigbee2mqtt websocket message.
    values: HashMap<String, serde_json::Value>,
}

impl ConfigClause {
    /// If the received message matches the conditions of this clause,
    /// return the values that we should set on the device.
    pub(crate) fn update_for(
        &self,
        payload: &HashMap<String, serde_json::Value>,
    ) -> Option<(String, HashMap<String, serde_json::Value>)> {
        if self
            .condition
            .iter()
            .all(|(k, v)| payload.get(k) == Some(v))
        {
            Some((
                self.name.as_ref().cloned().unwrap_or_default(),
                self.values.clone(),
            ))
        } else {
            None
        }
    }
}

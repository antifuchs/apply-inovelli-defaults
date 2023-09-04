use anyhow::Context;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
/// A configuration structure for this tool.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
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

pub fn load(path: &PathBuf) -> anyhow::Result<Vec<ConfigClause>> {
    let config_file = File::open(path).with_context(|| "Opening file")?;
    let mut config: serde_yaml::Value = serde_yaml::from_reader(config_file)
        .with_context(|| "Loading file, yaml may be invalid")?;
    config.apply_merge().with_context(|| "Merging yaml")?;
    serde_yaml::from_value(config).with_context(|| "Parsing file, unexpected format")
}

#[cfg(test)]
mod tests {
    use crate::config::*;

    #[test]
    fn valid_config() {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/config/valid.yaml");

        let expected = vec![ConfigClause {
            name: Some("test_valid".to_string()),
            condition: BTreeMap::from_iter(vec![(
                "mode".to_string(),
                serde_json::Value::String("test".to_string()),
            )]),
            values: HashMap::from_iter(vec![(
                "test".to_string(),
                serde_json::Value::Number(serde_json::Number::from(0)),
            )]),
        }];

        assert_eq!(load(&path).unwrap(), expected);
    }

    #[test]
    fn invalid_config_format() {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/config/invalid_format.yaml");

        assert!(matches!(load(&path), Err(t) if t.to_string().contains("unexpected format")));
    }
}

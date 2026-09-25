//! Validated JSON parsing, formatting, and value access.

use crate::{Error, Result};
pub use serde_json::Value;

pub fn parse(text: &str) -> Result<Value> {
    serde_json::from_str(text).map_err(|error| Error::new("json.parse", error))
}

pub fn compact(value: &Value) -> Result<String> {
    serde_json::to_string(value).map_err(|error| Error::new("json.compact", error))
}

pub fn pretty(value: &Value) -> Result<String> {
    serde_json::to_string_pretty(value).map_err(|error| Error::new("json.pretty", error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trip_preserves_data() {
        let value = parse(r#"{"name":"Adamantium","stable":false}"#).unwrap();
        assert_eq!(value["name"], "Adamantium");
        assert_eq!(parse(&compact(&value).unwrap()).unwrap(), value);
        assert!(parse("{").is_err());
    }
}

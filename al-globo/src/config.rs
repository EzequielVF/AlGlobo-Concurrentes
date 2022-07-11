use serde_json;
use serde_json::Value;

pub fn read_config(path: &str) -> Value {
    let text = std::fs::read_to_string(path).unwrap();
    let config = serde_json::from_str::<Value>(&text).unwrap();

    config
}

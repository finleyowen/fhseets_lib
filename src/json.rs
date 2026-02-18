use serde_json::Value;

/// Simple type that enforces a borrowing `to_json` method.
pub trait ToJson {
    fn to_json(&self) -> Value;
}

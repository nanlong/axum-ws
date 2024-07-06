use serde_json::Value;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Payload(Value);

impl From<Value> for Payload {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl From<Payload> for Value {
    fn from(payload: Payload) -> Self {
        payload.0
    }
}

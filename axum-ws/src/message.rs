use crate::{event::Event, payload::Payload, topic::Topic, websocket_error::WebSocketError};
use axum::extract::ws;
use derive_builder::Builder;
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, Default, Builder)]
pub(crate) struct Message {
    #[builder(setter(into, strip_option), default)]
    pub(crate) join_ref: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub(crate) message_ref: Option<String>,
    #[builder(setter(into), default)]
    pub(crate) topic: Topic,
    #[builder(setter(into))]
    pub(crate) event: Event,
    #[builder(setter(into))]
    pub(crate) payload: Payload,
}

impl Message {
    pub(crate) fn builder() -> MessageBuilder {
        MessageBuilder::default()
    }

    pub(crate) fn merge(&mut self, other: &Message) {
        self.join_ref.clone_from(&other.join_ref);
        self.message_ref.clone_from(&other.message_ref);
        self.topic.clone_from(&other.topic);
    }
}

impl TryFrom<&str> for Message {
    type Error = WebSocketError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let message: Vec<Value> = serde_json::from_str(value)?;

        if message.len() != 5 {
            return Err(Self::Error::InvalidMessage(value.to_string()));
        }

        let join_ref = message[0].as_str().map(|s| s.to_string());
        let message_ref = message[1].as_str().map(|s| s.to_string());
        let topic = message[2]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Self::Error::InvalidMessage("topic is required".to_string()))?
            .as_str()
            .into();
        let event = message[3]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Self::Error::InvalidMessage("event is required".to_string()))?
            .as_str()
            .into();
        let payload = message[4].clone().into();

        Ok(Self {
            join_ref,
            message_ref,
            topic,
            event,
            payload,
        })
    }
}

impl TryFrom<String> for Message {
    type Error = WebSocketError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let join_ref = self
            .join_ref
            .clone()
            .map(Value::String)
            .unwrap_or(Value::Null);
        let message_ref = self
            .message_ref
            .clone()
            .map(Value::String)
            .unwrap_or(Value::Null);
        let topic: Value = self.topic.clone().into();
        let event: Value = self.event.clone().into();
        let payload: Value = self.payload.clone().into();

        let message = serde_json::json!([join_ref, message_ref, topic, event, payload]);

        write!(f, "{}", message)
    }
}

impl From<Message> for ws::Message {
    fn from(message: Message) -> Self {
        ws::Message::Text(message.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handle::Response;
    use serde_json::json;

    #[test]
    fn message_heartbeat_should_work() {
        let message = Message::builder()
            .event("heartbeat")
            .payload(Response::Ok(json!({})))
            .build()
            .unwrap();

        assert_eq!(message.event, Event::Heartbeat);
        assert_eq!(message.payload, Response::Ok(json!({})).into());
    }

    #[test]
    fn message_ok_should_work() {
        let message = Message::builder()
            .event("reply")
            .payload(Response::Ok(json!({"test": "ok"})))
            .build()
            .unwrap();

        assert_eq!(message.event, Event::Reply);
        assert_eq!(
            message.payload,
            Response::Ok(json!({ "test": "ok" })).into()
        );
    }

    #[test]
    fn message_err_should_work() {
        let message = Message::builder()
            .event("reply")
            .payload(Response::Err(json!({"test": "err"})))
            .build()
            .unwrap();

        assert_eq!(message.event, Event::Reply);
        assert_eq!(
            message.payload,
            Response::Err(json!({ "test": "err" })).into()
        );
    }

    #[test]
    fn message_close_should_work() {
        let message = Message::builder()
            .event("close")
            .payload(Response::Empty)
            .build()
            .unwrap();

        assert_eq!(message.event, Event::Close);
        assert_eq!(message.payload, Response::Empty.into());
    }
}

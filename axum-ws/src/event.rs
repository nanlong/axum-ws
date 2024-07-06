use serde_json::Value;
use std::fmt;

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub enum Event {
    Join,
    Leave,
    Close,
    #[default]
    Reply,
    Heartbeat,
    Custom(String),
}

impl From<&str> for Event {
    fn from(s: &str) -> Self {
        match s {
            "phx_join" => Self::Join,
            "phx_leave" => Self::Leave,
            "phx_close" => Self::Close,
            "close" => Self::Close,
            "phx_reply" => Self::Reply,
            "reply" => Self::Reply,
            "heartbeat" => Self::Heartbeat,
            custom => Self::Custom(custom.to_string()),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::Join => write!(f, "phx_join"),
            Event::Leave => write!(f, "phx_leave"),
            Event::Close => write!(f, "phx_close"),
            Event::Reply => write!(f, "phx_reply"),
            Event::Heartbeat => write!(f, "heartbeat"),
            Event::Custom(custom) => write!(f, "{}", custom),
        }
    }
}

impl From<Event> for Value {
    fn from(event: Event) -> Self {
        Value::String(event.to_string())
    }
}

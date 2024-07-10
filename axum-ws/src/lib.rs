use std::sync::Arc;
use tokio::sync::Mutex;

mod assigns;
mod channel;
mod event;
mod handler;
mod message;
mod payload;
mod socket;
mod topic;
mod user_id;
mod websocket;
mod websocket_error;
mod websocket_state;

pub use channel::Channel;
pub use payload::Payload;
pub use topic::Topic;
pub use websocket::WebSocket;

pub type Socket = Arc<Mutex<socket::Socket>>;

use crate::{assigns::Assigns, message::Message, topic::Topic, websocket_state::WEBSOCKET_STATE};
use anyhow::Result;

#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct Socket {
    pub(crate) id: String,           // 用于标记socket的唯一id
    pub(crate) joined: bool,         // 当channel成功加入后，joined为true
    pub assigns: Assigns,            // 存储用户自定义的数据
    pub(crate) topic: Option<Topic>, // 不同的topic对应不同的channel
    pub(crate) message: Option<Message>,
}

impl Socket {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Default::default()
        }
    }

    pub(crate) fn set_message(&mut self, message: Message) {
        self.message = Some(message);
    }

    pub(crate) async fn push_message(&self, mut message: Message) -> Result<()> {
        if let Some(tx) = WEBSOCKET_STATE.get_sender(&self.id) {
            if let Some(m) = self.message.as_ref() {
                message.merge(m);
            }

            tx.send(message).await?;
        }

        Ok(())
    }
}

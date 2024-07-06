use crate::{assigns::Assigns, message::Message, topic::Topic, websocket_state::WEBSOCKET_STATE};
use anyhow::Result;

#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct Socket {
    pub id: String,           // 用于标记socket的唯一id
    pub joined: bool,         // 当channel成功加入后，joined为true
    pub assigns: Assigns,     // 存储用户自定义的数据
    pub topic: Option<Topic>, // 不同的topic对应不同的channel
    pub message: Option<Message>,
}

impl Socket {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Default::default()
        }
    }

    pub async fn push(&self, mut message: Message) -> Result<()> {
        if let Some(tx) = WEBSOCKET_STATE.get_sender(&self.id) {
            if let Some(m) = self.message.as_ref() {
                message.merge(m);
            }

            tx.send(message).await?;
        }

        Ok(())
    }

    pub(crate) fn set_message(&mut self, message: Message) {
        self.message = Some(message);
    }
}

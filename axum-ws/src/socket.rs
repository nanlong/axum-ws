use crate::{
    assigns::Assigns,
    handler::IntoResponse,
    message::Message,
    topic::Topic,
    websocket_state::{do_broadcast, WEBSOCKET_STATE},
};
use anyhow::Result;
use serde_json::Value;

#[derive(Debug, Default, Clone)]

pub struct Socket {
    pub(crate) id: String,
    pub(crate) joined: bool,
    pub(crate) path: String,
    pub(crate) topic: Option<Topic>,
    pub(crate) message: Option<Message>,
    pub assigns: Assigns,
}

impl Socket {
    pub fn new(id: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            ..Default::default()
        }
    }

    pub(crate) fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }

    pub(crate) fn set_topic(&mut self, topic: Topic) {
        self.topic = Some(topic);
    }

    pub(crate) fn set_joined(&mut self, joined: bool) {
        self.joined = joined;
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

    pub async fn push(&self, event: &str, data: Result<Value>) -> Result<()> {
        let response = data.into_response();
        let payload: Value = response.into();
        let mut message = Message::builder()
            .event(event)
            .payload(payload)
            .build()
            .unwrap();

        if let Some(m) = self.message.as_ref() {
            message.merge(m);
        }

        if let Some(tx) = WEBSOCKET_STATE.get_sender(&self.id) {
            tx.send(message).await?;
        }

        Ok(())
    }

    pub async fn broadcast(&self, event: &str, data: Result<Value>) -> Result<()> {
        do_broadcast(
            None,
            Some(&self.path),
            self.topic.as_ref(),
            event,
            data,
            self.message.as_ref(),
        )
        .await
    }

    pub async fn broadcast_from(
        &self,
        user_id: &str,
        event: &str,
        data: Result<Value>,
    ) -> Result<()> {
        do_broadcast(
            Some(user_id),
            Some(&self.path),
            self.topic.as_ref(),
            event,
            data,
            self.message.as_ref(),
        )
        .await
    }
}

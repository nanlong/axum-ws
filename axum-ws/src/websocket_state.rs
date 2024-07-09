use crate::{handler::IntoResponse, message::Message, topic::Topic, user_id::UserId};
use anyhow::Result;
use dashmap::DashMap;
use serde_json::Value;
use std::{any::TypeId, borrow::Borrow, collections::HashSet, hash::Hash};
use tokio::sync::mpsc::Sender;

lazy_static::lazy_static!(
    pub(crate) static ref WEBSOCKET_STATE: WebSocketState = WebSocketState::default();
);

#[derive(Default)]
pub(crate) struct WebSocketState {
    path: DashMap<TypeId, String>,
    sender: DashMap<UserId, Sender<Message>>,
    users: DashMap<(String, Topic), HashSet<UserId>>,
}

impl WebSocketState {
    pub fn insert_path<T>(&self, path: impl Into<String>)
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.path.insert(type_id, path.into());
    }

    pub fn get_path<T>(&self) -> Option<String>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.path.get(&type_id).map(|path| path.value().clone())
    }

    pub fn insert_sender<K>(&self, key: K, val: Sender<Message>)
    where
        K: Into<UserId>,
    {
        self.sender.insert(key.into(), val);
    }

    pub fn get_sender<Q>(&self, key: &Q) -> Option<Sender<Message>>
    where
        Q: ?Sized + Hash + Eq,
        UserId: Borrow<Q>,
    {
        self.sender.get(key).map(|entry| entry.value().clone())
    }

    pub fn remove_sender<Q>(&self, key: &Q) -> Option<Sender<Message>>
    where
        Q: ?Sized + Hash + Eq,
        UserId: Borrow<Q>,
    {
        self.sender.remove(key).map(|(_, sender)| sender)
    }

    pub fn insert_user(&self, key: (String, Topic), entry: UserId) {
        self.users.entry(key).or_default().insert(entry);
    }

    pub fn get_users(&self, key: &(String, Topic)) -> Option<HashSet<UserId>> {
        self.users
            .get(key)
            .map(|entry| entry.value().iter().cloned().collect())
    }

    pub fn remove_user(&self, key: &(String, Topic), entry: &UserId) -> bool {
        if let Some(mut users) = self.users.get_mut(key) {
            users.remove(entry)
        } else {
            false
        }
    }

    pub fn clearn_user(&self, entry: &UserId) {
        self.remove_sender(entry);

        for mut users in self.users.iter_mut() {
            users.value_mut().remove(entry);
        }
    }
}

pub(crate) async fn do_broadcast(
    exclude_user: Option<&str>,
    path: Option<&String>,
    topic: Option<&Topic>,
    event: &str,
    data: Result<Value>,
    prev_message: Option<&Message>,
) -> Result<()> {
    let response = data.into_response();
    let payload: Value = response.into();
    let mut message = Message::builder()
        .topic(topic.cloned().unwrap_or_default())
        .event(event)
        .payload(payload)
        .build()
        .unwrap();

    if let Some(m) = prev_message {
        message.merge(m);
    }

    if let (Some(path), Some(topic)) = (path, topic) {
        if let Some(users) = WEBSOCKET_STATE.get_users(&(path.clone(), topic.clone())) {
            for user in users.iter() {
                if exclude_user.map_or(true, |id| user.as_str() != id) {
                    if let Some(tx) = WEBSOCKET_STATE.get_sender(user) {
                        tx.send(message.clone()).await?;
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct UserSocket;

    #[test]
    fn websocket_state_with_path_should_work() {
        WEBSOCKET_STATE.insert_path::<UserSocket>("/socket".to_string());

        assert_eq!(
            WEBSOCKET_STATE.get_path::<UserSocket>().unwrap(),
            "/socket".to_string()
        );
    }
}

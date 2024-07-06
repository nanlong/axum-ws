use crate::{message::Message, topic::Topic, user_id::UserId};
use dashmap::DashMap;
use std::{any::TypeId, borrow::Borrow, collections::HashSet, hash::Hash};
use tokio::sync::mpsc::Sender;

lazy_static::lazy_static!(
    pub(crate) static ref WEBSOCKET_STATE: WebSocketState = WebSocketState::default();
);

#[derive(Default)]
pub(crate) struct WebSocketState {
    path: DashMap<TypeId, String>,
    sender: DashMap<UserId, Sender<Message>>,
    users: DashMap<Topic, HashSet<UserId>>,
}

#[allow(dead_code)]
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

    pub fn insert_user<K, V>(&self, key: K, entry: V)
    where
        K: Into<Topic>,
        V: Into<UserId>,
    {
        self.users
            .entry(key.into())
            .or_default()
            .insert(entry.into());
    }

    pub fn get_users<Q>(&self, key: &Q) -> Option<HashSet<UserId>>
    where
        Q: ?Sized + Hash + Eq,
        Topic: Borrow<Q>,
    {
        self.users
            .get(key)
            .map(|entry| entry.value().iter().cloned().collect())
    }

    pub fn remove_user<Q, E>(&mut self, key: &Q, entry: &E) -> bool
    where
        Q: ?Sized + Hash + Eq,
        E: ?Sized + Hash + Eq,
        Topic: Borrow<Q>,
        UserId: Borrow<E>,
    {
        if let Some(mut users) = self.users.get_mut(key) {
            users.remove(entry)
        } else {
            false
        }
    }
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

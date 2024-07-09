use crate::{
    channel::Channel,
    event::Event,
    handler::{Connect, ConnectWrapper, Id, IdWrapper},
    handler::{IntoResponse, Response},
    message::Message,
    socket,
    topic::Topic,
    websocket_error::WebSocketError,
    websocket_state::{do_broadcast, WEBSOCKET_STATE},
    Socket,
};
use anyhow::Result;
use axum::{
    extract::{ws, Query, WebSocketUpgrade},
    routing::get,
    Extension, Router,
};
use futures::{Future, SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{collections::HashMap, marker::PhantomData, sync::Arc};
use tokio::sync::{mpsc, Mutex};

const USER_BUFFER_SIZE: usize = 1024;

#[derive(Default)]

pub struct WebSocket<T> {
    path: String,
    channels: HashMap<Topic, Channel>,
    connect: Option<Box<dyn Connect + Send + Sync>>,
    id: Option<Box<dyn Id + Send + Sync>>,
    _tag: PhantomData<T>,
}

#[allow(dead_code)]
impl<T> WebSocket<T>
where
    T: Default + Send + Sync + 'static,
{
    pub fn new(path: impl Into<String>) -> Self {
        let path = path.into();
        WEBSOCKET_STATE.insert_path::<T>(path.clone());

        Self {
            path,
            ..Default::default()
        }
    }

    pub fn channel(mut self, topic: impl Into<Topic>, channel: Channel) -> Self {
        self.channels.insert(topic.into(), channel);
        self
    }

    fn get_channel(&self, topic: &Topic) -> Option<&Channel> {
        for (t, c) in &self.channels {
            if t.is_match(topic) {
                return Some(c);
            }
        }

        None
    }

    async fn upgrade(
        websocket_upgrade: WebSocketUpgrade,
        Query(params): Query<Value>,
        Extension(websocket): Extension<Arc<WebSocket<T>>>,
    ) -> axum::response::Response {
        let mut user_id = nanoid::nanoid!();
        let path = websocket.path.clone();
        let socket = Arc::new(Mutex::new(socket::Socket::new(user_id.clone(), path)));
        let shared_socket = socket.clone();

        if let Some(connect) = websocket.connect.as_ref() {
            let res = connect.call(params, shared_socket.clone()).await;

            if res.status().is_client_error() || res.status().is_server_error() {
                return res;
            }
        }

        if let Some(id) = websocket.id.as_ref() {
            if let Some(gen_id) = id.call(shared_socket.clone()).await {
                let mut socket = shared_socket.lock().await;
                socket.set_id(gen_id.clone());
                user_id = gen_id;
            }
        }

        websocket_upgrade.on_upgrade(|axum_websocket| async move {
            let (mut sender, mut receiver) = axum_websocket.split();
            let (tx, mut rx) = mpsc::channel(USER_BUFFER_SIZE);

            WEBSOCKET_STATE.insert_sender(user_id.clone(), tx);

            let mut recv_task = tokio::spawn(async move {
                let mut sockets = HashMap::<Topic, Socket>::new();

                while let Some(Ok(ws::Message::Text(message))) = receiver.next().await {
                    let message: Message = message.try_into()?;

                    match message.event {
                        Event::Join => {
                            {
                                let mut socket = shared_socket.lock().await;
                                socket.set_message(message.clone());
                            }

                            let topic = message.topic.clone();

                            if let Some(channel) = websocket.get_channel(&topic) {
                                if let Some(join) = channel.join.as_ref() {
                                    let res = join
                                        .call(
                                            topic.clone(),
                                            message.payload.clone(),
                                            shared_socket.clone(),
                                        )
                                        .await;
                                    let mut socket = shared_socket.lock().await;

                                    if res.is_ok() {
                                        sockets.insert(
                                            topic.clone(),
                                            Arc::new(Mutex::new(socket.clone())),
                                        );
                                        WEBSOCKET_STATE.insert_user(
                                            (websocket.path.clone(), topic.clone()),
                                            socket.id.clone().into(),
                                        );
                                        socket.set_joined(true);
                                        socket.set_topic(topic.clone());
                                    }

                                    let payload: Value = res.into_response().into();
                                    let message = Message::builder()
                                        .event("reply")
                                        .payload(payload)
                                        .build()
                                        .unwrap();

                                    socket.push_message(message).await?;
                                }
                            } else {
                                let message = Message::builder()
                                    .event("reply")
                                    .payload(Response::Err("unmatched topic".into()))
                                    .build()
                                    .unwrap();

                                let socket = shared_socket.lock().await;
                                socket.push_message(message).await?;
                            }
                        }
                        Event::Leave => {
                            let mut socket = shared_socket.lock().await;
                            let topic = message.topic.clone();
                            socket.set_message(message.clone());

                            let message = Message::builder()
                                .event("reply")
                                .payload(Response::Ok(json!({})))
                                .build()
                                .unwrap();

                            socket.push_message(message).await?;

                            let user_id = socket.id.clone().into();

                            WEBSOCKET_STATE
                                .remove_user(&(websocket.path.clone(), topic.clone()), &user_id);

                            sockets.remove(&topic);

                            let message = Message::builder()
                                .event("close")
                                .payload(Response::NoReply)
                                .build()
                                .unwrap();

                            socket.push_message(message).await?;
                        }
                        Event::Heartbeat => {
                            let mut socket = shared_socket.lock().await;
                            socket.set_message(message.clone());

                            let message = Message::builder()
                                .event("heartbeat")
                                .payload(Response::Ok(json!({})))
                                .build()
                                .unwrap();

                            socket.push_message(message).await?;
                        }
                        Event::Custom(ref event) => {
                            if let Some(socket) = sockets.get(&message.topic) {
                                {
                                    let mut socket = socket.lock().await;
                                    socket.set_message(message.clone());
                                }

                                if let Some(channel) = websocket.get_channel(&message.topic) {
                                    if let Some(handler) = channel.handler.get(event) {
                                        let res = handler
                                            .call(message.payload.clone(), socket.clone())
                                            .await;

                                        if res != Response::NoReply {
                                            let payload: Value = res.into_response().into();
                                            let message = Message::builder()
                                                .event("reply")
                                                .payload(payload)
                                                .build()
                                                .unwrap();

                                            let socket = socket.lock().await;
                                            socket.push_message(message).await?;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }

                Ok::<_, WebSocketError>(())
            });

            let mut send_task = tokio::spawn(async move {
                while let Some(message) = rx.recv().await {
                    sender.send(message.into()).await?;
                }

                Ok::<_, WebSocketError>(())
            });

            tokio::select! {
                _ = (&mut send_task) => recv_task.abort(),
                _ = (&mut recv_task) => send_task.abort(),
            };

            let user_id = socket.lock().await.id.clone().into();
            WEBSOCKET_STATE.clearn_user(&user_id);
        })
    }

    async fn broadcast(topic: &str, event: &str, data: Result<Value>) -> Result<()> {
        if let Some(path) = WEBSOCKET_STATE.get_path::<T>() {
            let topic: Topic = topic.into();

            do_broadcast(None, Some(&path), Some(&topic), event, data, None).await?;
        }
        Ok(())
    }

    async fn broadcast_from(
        user_id: &str,
        topic: &str,
        event: &str,
        data: Result<Value>,
    ) -> Result<()> {
        if let Some(path) = WEBSOCKET_STATE.get_path::<T>() {
            let topic: Topic = topic.into();

            do_broadcast(Some(user_id), Some(&path), Some(&topic), event, data, None).await?;
        }
        Ok(())
    }

    fn connect<F, Fut, Res>(mut self, connect: F) -> Self
    where
        F: Fn(Value, Socket) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Res> + Send + 'static,
        Res: axum::response::IntoResponse,
    {
        self.connect = Some(Box::new(ConnectWrapper::new(move |params, socket| {
            let connect = connect.clone();

            Box::pin(async move {
                let res = connect(params, socket).await;
                res.into_response()
            })
        })) as Box<dyn Connect + Send + Sync>);
        self
    }

    fn id<F, Fut>(mut self, id: F) -> Self
    where
        F: Fn(Socket) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<String>> + Send + 'static,
    {
        self.id = Some(Box::new(IdWrapper::new(move |socket| Box::pin(id(socket))))
            as Box<dyn Id + Send + Sync>);
        self
    }
}

impl<T> From<WebSocket<T>> for Router
where
    T: Default + Send + Sync + 'static,
{
    fn from(websocket: WebSocket<T>) -> Self {
        Router::new()
            .route(
                &format!("{}/websocket", websocket.path),
                get(WebSocket::<T>::upgrade),
            )
            .layer(Extension(Arc::new(websocket)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn websocket_callback_should_work() {
        async fn test_connect(_params: Value, socket: Socket) {
            let mut socket = socket.lock().await;
            socket.assigns.insert("test", 1);
        }

        async fn test_id(_socket: Socket) -> Option<String> {
            Some("test".to_string())
        }

        let websocket = WebSocket::<String>::new("/test")
            .connect(test_connect)
            .id(test_id);

        // let connect = websocket.connect.unwrap();
        // let response = connect.call(json!({}), Socket::default()).await;
        // assert_eq!(response, ());

        let id = websocket.id.unwrap();
        let response = id.call(Socket::default()).await;
        assert_eq!(response, Some("test".to_string()));
    }
}

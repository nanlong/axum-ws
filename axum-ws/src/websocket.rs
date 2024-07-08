use crate::{
    channel::Channel,
    event::Event,
    handler::Response,
    handler::{Connect, ConnectWrapper, Id, IdWrapper},
    message::Message,
    socket::Socket,
    topic::Topic,
    websocket_error::WebSocketError,
    websocket_state::WEBSOCKET_STATE,
};
use anyhow::Result;
use axum::{
    extract::{ws, Query, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use futures::{Future, SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{collections::HashMap, marker::PhantomData, sync::Arc};
use tokio::sync::mpsc;

const USER_BUFFER_SIZE: usize = 1024;

#[derive(Default)]
#[allow(dead_code)]
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

    async fn upgrade(
        websocket_upgrade: WebSocketUpgrade,
        Query(_params): Query<Value>,
        Extension(_websocket): Extension<Arc<WebSocket<T>>>,
    ) -> impl IntoResponse {
        let user_id = nanoid::nanoid!();

        websocket_upgrade.on_upgrade(|axum_websocket| async move {
            let (mut sender, mut receiver) = axum_websocket.split();
            let (tx, mut rx) = mpsc::channel(USER_BUFFER_SIZE);
            let mut socket = Socket::new(user_id.clone());

            WEBSOCKET_STATE.insert_sender(user_id.clone(), tx);

            let mut recv_task = tokio::spawn(async move {
                let mut sockets = HashMap::<Topic, Socket>::new();

                while let Some(Ok(ws::Message::Text(message))) = receiver.next().await {
                    let message: Message = message.try_into()?;

                    match message.event {
                        Event::Join => {
                            let mut socket = socket.clone();
                            let topic = message.topic.clone();
                            socket.set_message(message.clone());

                            let message = Message::builder()
                                .event("reply")
                                .payload(Response::Ok(json!({})))
                                .build()
                                .unwrap();

                            socket.push_message(message).await?;

                            sockets.insert(topic, socket);
                        }
                        Event::Leave => {
                            let topic = message.topic.clone();
                            socket.set_message(message.clone());

                            let message = Message::builder()
                                .event("reply")
                                .payload(Response::Ok(json!({})))
                                .build()
                                .unwrap();

                            socket.push_message(message).await?;

                            // 成功离开channel后
                            let message = Message::builder()
                                .event("close")
                                .payload(Response::NoReply)
                                .build()
                                .unwrap();

                            socket.push_message(message).await?;

                            sockets.remove(&topic);
                        }
                        Event::Heartbeat => handle_heartbeat(&mut socket, &message).await?,
                        Event::Custom(_) => {
                            let socket = sockets.get_mut(&message.topic).unwrap_or(&mut socket);

                            socket.set_message(message.clone());
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

            WEBSOCKET_STATE.remove_sender(&user_id);
        })
    }

    fn connect<F, Fut>(mut self, connect: F) -> Self
    where
        F: Fn(Value, Socket) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.connect = Some(Box::new(ConnectWrapper::new(move |params, socket| {
            Box::pin(connect(params, socket))
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

async fn handle_heartbeat(socket: &mut Socket, message: &Message) -> Result<()> {
    socket.set_message(message.clone());

    let message = Message::builder()
        .event("heartbeat")
        .payload(Response::Ok(json!({})))
        .build()
        .unwrap();

    socket.push_message(message).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn websocket_callback_should_work() {
        async fn test_connect(_params: Value, _socket: Socket) {}

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

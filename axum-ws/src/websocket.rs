use crate::{
    channel::Channel, event::Event, handler::Response, message::Message, socket::Socket,
    topic::Topic, websocket_error::WebSocketError, websocket_state::WEBSOCKET_STATE,
};
use anyhow::Result;
use axum::{
    extract::{ws, Query, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{collections::HashMap, marker::PhantomData, sync::Arc};
use tokio::sync::mpsc;

const USER_BUFFER_SIZE: usize = 1024;

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct WebSocket<T> {
    path: String,
    channels: HashMap<Topic, Channel>,
    _tag: PhantomData<T>,
}

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

    async fn connect(
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

                            socket.push(message).await?;

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

                            socket.push(message).await?;

                            // 成功离开channel后
                            let message = Message::builder()
                                .event("close")
                                .payload(Response::Empty)
                                .build()
                                .unwrap();

                            socket.push(message).await?;

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
}

impl<T> From<WebSocket<T>> for Router
where
    T: Default + Send + Sync + 'static,
{
    fn from(websocket: WebSocket<T>) -> Self {
        Router::new()
            .route(
                &format!("{}/websocket", websocket.path),
                get(WebSocket::<T>::connect),
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

    socket.push(message).await?;

    Ok(())
}

use crate::{
    handler::{Handler, HandlerWrapper, IntoResponse, Join, JoinWrapper},
    payload::Payload,
    topic::Topic,
    Socket,
};
use anyhow::Result;
use futures::Future;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Default)]
pub struct Channel {
    pub(crate) join: Option<Box<dyn Join + Send + Sync>>,
    pub(crate) handler: HashMap<String, Box<dyn Handler + Send + Sync>>,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            join: None,
            handler: HashMap::new(),
        }
    }

    pub fn join<F, Fut, Res>(mut self, join: F) -> Self
    where
        F: Fn(Topic, Payload, Socket) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<Res>> + Send + 'static,
        Res: Into<Value>,
    {
        self.join = Some(Box::new(JoinWrapper::new(move |topic, payload, socket| {
            let join = join.clone();
            Box::pin(async move {
                let res = join(topic, payload, socket).await?;
                Ok(res.into())
            })
        })) as Box<dyn Join + Send + Sync>);
        self
    }

    pub fn handler<F, Fut, Res>(mut self, event: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Payload, Socket) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Res> + Send + 'static,
        Res: IntoResponse,
    {
        let event = event.into();
        let handler = Box::new(HandlerWrapper::new(move |payload, socket| {
            let handler = handler.clone();
            Box::pin(async move {
                let res = handler(payload, socket).await;
                res.into_response()
            })
        })) as Box<dyn Handler + Send + Sync>;

        self.handler.insert(event, handler);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::Response;

    #[tokio::test]
    async fn channel_callback_should_work() {
        async fn join(_topic: Topic, _payload: Payload, socket: Socket) -> anyhow::Result<String> {
            let mut socket = socket.lock().await;
            socket.assigns.insert("test", 1);

            Ok("ok".to_string())
        }

        async fn event1(_payload: Payload, _socket: Socket) -> Response {
            Response::NoReply
        }

        async fn event2(_payload: Payload, _socket: Socket) {}

        let channel = Channel::new()
            .join(join)
            .handler("event1", event1)
            .handler("event2", event2);

        let socket = Socket::default();

        let join = channel.join.unwrap();
        join.call(Topic::default(), Payload::default(), socket.clone())
            .await
            .unwrap();
        // assert_eq!(socket.assigns.get("test"), Some(&1));

        let event1 = channel.handler.get("event1").unwrap();
        let response = event1.call(Payload::default(), Socket::default()).await;
        assert_eq!(response, Response::NoReply);

        let event2 = channel.handler.get("event2").unwrap();
        let response = event2.call(Payload::default(), Socket::default()).await;
        assert_eq!(response, Response::NoReply);

        let event1 = channel.handler.get("event1").unwrap();
        let response = event1.call(Payload::default(), Socket::default()).await;
        assert_eq!(response, Response::NoReply);
    }
}

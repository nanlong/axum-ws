use crate::{
    handler::{Handler, HandlerWrapper, IntoResponse, Join, JoinWrapper},
    payload::Payload,
    socket::Socket,
    topic::Topic,
};
use futures::Future;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct Channel {
    join: Option<Box<dyn Join + Send + Sync>>,
    handler: HashMap<String, Box<dyn Handler + Send + Sync>>,
}

#[allow(dead_code)]
impl Channel {
    pub fn new() -> Self {
        Self {
            join: None,
            handler: HashMap::new(),
        }
    }

    pub fn join<F, Fut>(mut self, join: F) -> Self
    where
        F: Fn(Topic, Payload, Socket) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.join = Some(Box::new(JoinWrapper::new(move |topic, payload, socket| {
            Box::pin(join(topic, payload, socket))
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
        async fn join(_topic: Topic, _payload: Payload, mut socket: Socket) {
            socket.assigns.insert("test", 1);
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
            .await;
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

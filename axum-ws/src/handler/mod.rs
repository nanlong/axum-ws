use crate::{payload::Payload, topic::Topic, Socket};
use anyhow::Result;
use futures::future::BoxFuture;
use serde_json::Value;

mod into_response;
mod response;

pub(crate) use self::{into_response::IntoResponse, response::Response};

#[allow(dead_code)]
pub(crate) trait Connect: Send + Sync {
    fn call(&self, params: Value, socket: Socket) -> BoxFuture<'static, axum::response::Response>;
}

pub(crate) struct ConnectWrapper<F> {
    handler: F,
}

impl<F> Connect for ConnectWrapper<F>
where
    F: Fn(Value, Socket) -> BoxFuture<'static, axum::response::Response> + Send + Sync + 'static,
{
    fn call(&self, params: Value, socket: Socket) -> BoxFuture<'static, axum::response::Response> {
        (self.handler)(params, socket)
    }
}

#[allow(dead_code)]
impl<F> ConnectWrapper<F>
where
    F: Fn(Value, Socket) -> BoxFuture<'static, axum::response::Response> + Send + Sync + 'static,
{
    pub fn new(handler: F) -> Self {
        ConnectWrapper { handler }
    }
}

#[allow(dead_code)]
pub(crate) trait Id: Send + Sync {
    fn call(&self, socket: Socket) -> BoxFuture<'static, Option<String>>;
}

pub(crate) struct IdWrapper<F> {
    handler: F,
}

impl<F> Id for IdWrapper<F>
where
    F: Fn(Socket) -> BoxFuture<'static, Option<String>> + Send + Sync + 'static,
{
    fn call(&self, socket: Socket) -> BoxFuture<'static, Option<String>> {
        (self.handler)(socket)
    }
}

#[allow(dead_code)]
impl<F> IdWrapper<F>
where
    F: Fn(Socket) -> BoxFuture<'static, Option<String>> + Send + Sync + 'static,
{
    pub fn new(handler: F) -> Self {
        IdWrapper { handler }
    }
}

#[allow(dead_code)]
pub(crate) trait Join: Send + Sync {
    fn call(
        &self,
        topic: Topic,
        payload: Payload,
        socket: Socket,
    ) -> BoxFuture<'static, Result<Value>>;
}

pub(crate) struct JoinWrapper<F> {
    handler: F,
}

impl<F> Join for JoinWrapper<F>
where
    F: Fn(Topic, Payload, Socket) -> BoxFuture<'static, Result<Value>> + Send + Sync + 'static,
{
    fn call(
        &self,
        topic: Topic,
        payload: Payload,
        socket: Socket,
    ) -> BoxFuture<'static, Result<Value>> {
        (self.handler)(topic, payload, socket)
    }
}

#[allow(dead_code)]
impl<F> JoinWrapper<F>
where
    F: Fn(Topic, Payload, Socket) -> BoxFuture<'static, Result<Value>> + Send + Sync + 'static,
{
    pub fn new(handler: F) -> Self {
        JoinWrapper { handler }
    }
}

#[allow(dead_code)]
pub(crate) trait Handler: Send + Sync {
    fn call(&self, payload: Payload, socket: Socket) -> BoxFuture<'static, Response>;
}

pub(crate) struct HandlerWrapper<F> {
    handler: F,
}

impl<F> Handler for HandlerWrapper<F>
where
    F: Fn(Payload, Socket) -> BoxFuture<'static, Response> + Send + Sync + 'static,
{
    fn call(&self, payload: Payload, socket: Socket) -> BoxFuture<'static, Response> {
        (self.handler)(payload, socket)
    }
}

#[allow(dead_code)]
impl<F> HandlerWrapper<F>
where
    F: Fn(Payload, Socket) -> BoxFuture<'static, Response> + Send + Sync + 'static,
{
    pub fn new(handler: F) -> Self {
        HandlerWrapper { handler }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    async fn test_connect(_payload: Payload, _socket: Socket) -> Response {
        Response::NoReply
    }

    struct HandlerStore {
        pub handler: Box<dyn Handler>,
    }

    #[tokio::test]
    async fn connect_wrapper_should_work() {
        let connect = Box::new(HandlerWrapper::new(|payload, socket| {
            Box::pin(async move {
                let res = test_connect(payload, socket).await;
                res.into_response()
            })
        })) as Box<dyn Handler>;

        let store = HandlerStore { handler: connect };

        let payload = Payload::from(json!({"test": "ok"}));
        let socket = Socket::default();

        let response = store.handler.call(payload.clone(), socket.clone()).await;

        assert_eq!(response, Response::NoReply);
    }
}

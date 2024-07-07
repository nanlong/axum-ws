use crate::{payload::Payload, socket::Socket, topic::Topic};
use futures::future::BoxFuture;
use serde_json::Value;

mod into_response;
mod response;

pub(crate) use self::response::Response;

#[allow(dead_code)]
pub(crate) trait Connect: Send + Sync {
    fn call(&self, params: Value, socket: Socket) -> BoxFuture<'static, Response>;
}

pub(crate) struct ConnectWrapper<F> {
    handler: F,
}

impl<F> Connect for ConnectWrapper<F>
where
    F: Fn(Value, Socket) -> BoxFuture<'static, Response> + Send + Sync + 'static,
{
    fn call(&self, params: Value, socket: Socket) -> BoxFuture<'static, Response> {
        (self.handler)(params, socket)
    }
}

#[allow(dead_code)]
impl<F> ConnectWrapper<F>
where
    F: Fn(Value, Socket) -> BoxFuture<'static, Response> + Send + Sync + 'static,
{
    fn new(handler: F) -> Self {
        ConnectWrapper { handler }
    }
}

#[allow(dead_code)]
pub(crate) trait Id: Send + Sync {
    fn call(&self, socket: Socket) -> BoxFuture<'static, Response>;
}

pub(crate) struct IdWrapper<F> {
    handler: F,
}

impl<F> Id for IdWrapper<F>
where
    F: Fn(Socket) -> BoxFuture<'static, Response> + Send + Sync + 'static,
{
    fn call(&self, socket: Socket) -> BoxFuture<'static, Response> {
        (self.handler)(socket)
    }
}

#[allow(dead_code)]
impl<F> IdWrapper<F>
where
    F: Fn(Value, Socket) -> BoxFuture<'static, Response> + Send + Sync + 'static,
{
    fn new(handler: F) -> Self {
        IdWrapper { handler }
    }
}

#[allow(dead_code)]
pub(crate) trait Join: Send + Sync {
    fn call(&self, topic: Topic, payload: Payload, socket: Socket) -> BoxFuture<'static, Response>;
}

pub(crate) struct JoinWrapper<F> {
    handler: F,
}

impl<F> Join for JoinWrapper<F>
where
    F: Fn(Topic, Payload, Socket) -> BoxFuture<'static, Response> + Send + Sync + 'static,
{
    fn call(&self, topic: Topic, payload: Payload, socket: Socket) -> BoxFuture<'static, Response> {
        (self.handler)(topic, payload, socket)
    }
}

#[allow(dead_code)]
impl<F> JoinWrapper<F>
where
    F: Fn(Topic, Payload, Socket) -> BoxFuture<'static, Response> + Send + Sync + 'static,
{
    fn new(handler: F) -> Self {
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
    fn new(handler: F) -> Self {
        HandlerWrapper { handler }
    }
}

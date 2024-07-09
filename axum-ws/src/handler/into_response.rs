use std::fmt::Display;

use super::Response;
use serde_json::Value;

#[allow(dead_code)]
pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::NoReply
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: Into<Value>,
    E: Display,
{
    fn into_response(self) -> Response {
        match self {
            Ok(v) => Response::Ok(v.into()),
            Err(e) => Response::Err(e.to_string().into()),
        }
    }
}

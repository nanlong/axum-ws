use std::fmt::Display;

use super::Response;
use serde_json::Value;

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
            Ok(v) => {
                let value = v.into();

                if value.is_null() {
                    Response::NoReply
                } else {
                    Response::Ok(value)
                }
            }
            Err(e) => match serde_json::from_str(&e.to_string()) {
                Ok(value) => Response::Err(value),
                Err(_) => Response::Err(e.to_string().into()),
            },
        }
    }
}

use serde_json::{json, Value};

use crate::payload::Payload;

#[derive(Debug, PartialEq)]
pub(crate) enum Response {
    Ok(Value),
    Err(Value),
    Empty,
    NoReply,
}

impl From<Response> for Value {
    fn from(response: Response) -> Value {
        match response {
            Response::Ok(value) => json!({"status": "ok", "response": value}),
            Response::Err(value) => json!({"status": "error", "response": value}),
            Response::Empty => json!({}),
            Response::NoReply => json!(null),
        }
    }
}

impl From<Response> for Payload {
    fn from(response: Response) -> Payload {
        let value: Value = response.into();
        value.into()
    }
}

#[allow(dead_code)]
pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::NoReply
    }
}

impl<T> IntoResponse for Result<T, T>
where
    T: Into<Value>,
{
    fn into_response(self) -> Response {
        match self {
            Ok(value) => Response::Ok(value.into()),
            Err(e) => Response::Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;
    use thiserror::Error;

    #[derive(Debug, Error)]
    struct TestError(String);

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl From<TestError> for Value {
        fn from(error: TestError) -> Value {
            json!(error.to_string())
        }
    }

    fn test_with_ok() -> impl IntoResponse {
        Ok("test")
    }

    fn test_with_error() -> impl IntoResponse {
        Err(())
    }

    fn test_with_custom_error() -> impl IntoResponse {
        Err(TestError("error".to_string()))
    }

    fn test_with_empty() -> impl IntoResponse {}

    #[test]
    fn test_into_response() {
        let response = test_with_ok();
        assert_eq!(response.into_response(), Response::Ok(json!("test")));

        let response = test_with_error();
        assert_eq!(response.into_response(), Response::Err(json!(())));

        let response = test_with_custom_error();
        assert_eq!(response.into_response(), Response::Err(json!("error")));

        let response = test_with_empty();
        assert_eq!(response.into_response(), Response::NoReply);
    }
}

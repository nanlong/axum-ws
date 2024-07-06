use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebSocketError {
    #[error("json error: {0}")]
    JsonSerializeError(#[from] serde_json::Error),

    #[error("invalid message: {0}")]
    InvalidMessage(String),

    #[error("app error: {0}")]
    AppError(#[from] axum::Error),

    #[error("app error: {0}")]
    AnyhowError(#[from] anyhow::Error),
}

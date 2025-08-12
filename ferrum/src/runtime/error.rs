use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("task canceled")]
    Canceled,

    #[error("timeout")]
    Timeout,

    #[error("queue is full")]
    QueueFull,

    #[error("service unavailable")]
    ServiceUnavailable,

    #[error("serialization error: {0}")]
    Serialize(String),

    #[error("deserialization error: {0}")]
    Deserialize(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<tokio::sync::oneshot::error::RecvError> for Error {
    fn from(_: tokio::sync::oneshot::error::RecvError) -> Self {
        Error::Canceled
    }
}

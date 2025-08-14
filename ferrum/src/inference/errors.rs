use thiserror::Error;

#[derive(Error, Debug)]
pub enum InferenceError {
    #[error("Inference error: {0}")]
    InferenceError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

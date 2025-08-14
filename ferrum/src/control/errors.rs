use thiserror::Error;

#[derive(Error, Debug)]
pub enum ControlError {
    #[error("Control error: {0}")]
    ControlError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

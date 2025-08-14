use thiserror::Error;

#[derive(Error, Debug)]
pub enum EnvError {
    #[error("Environment error: {0}")]
    EnvError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

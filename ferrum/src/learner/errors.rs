use thiserror::Error;

#[derive(Error, Debug)]
pub enum LearnerError {
    #[error("Learner error: {0}")]
    LearnerError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

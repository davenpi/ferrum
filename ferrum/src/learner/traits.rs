use super::errors::LearnerError;
use crate::env::TrajectoryShard;
use crate::inference::Inference;
use async_trait::async_trait;

#[async_trait]
pub trait Learner<O, A>: Send + Sync {
    fn client(&self) -> impl Inference<O, A>;
    async fn submit(&mut self, shards: Vec<TrajectoryShard<O, A>>) -> Result<(), LearnerError>;
    async fn update(
        &mut self,
    ) -> Result<(u64 /*new_version*/, String /*checkpoint_uri*/), LearnerError>;
}

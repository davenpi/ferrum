use super::errors::InferenceError;
use async_trait::async_trait;

#[async_trait]
pub trait InferenceClient<O, A>: Clone + Send + Sync + 'static {
    async fn infer(&self, version_id: u64, obs: Vec<O>)
    -> Result<InferResponse<A>, InferenceError>;
    async fn init_communication(&mut self, config: CommConfig) -> Result<(), InferenceError>;
    async fn update_weights(
        &mut self,
        checkpoint_uri: String,
        version_id: u64,
    ) -> Result<(), InferenceError>;
}

pub struct InferResponse<A> {
    pub actions: Vec<A>,
    pub logprobs: Vec<f32>, // for importance sampling
}

/// Configuration for communication with the Learner
pub struct CommConfig {}

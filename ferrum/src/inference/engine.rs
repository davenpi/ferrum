use crate::inference::types::*;
use async_trait::async_trait;

#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn generate(
        &self,
        input: InferenceEngineInput,
    ) -> Result<InferenceEngineOutput, InferenceError>;

    // Add optional tags parameter to match actual usage
    async fn wake_up(&self, tags: Option<Vec<String>>) -> Result<(), InferenceError>;

    // Add optional level parameter to match vLLM API
    async fn sleep(&self, level: Option<i32>) -> Result<(), InferenceError>;

    async fn init_weight_update_communicator(
        &self,
        master_addr: String,
        master_port: u16,
        rank_offset: usize,
        world_size: usize,
        group_name: String,
        backend: String,
        override_existing: bool,
    ) -> Result<(), InferenceError>;

    async fn update_named_weight(
        &self,
        request: NamedWeightUpdateRequest,
    ) -> Result<(), InferenceError>;

    async fn teardown(&self) -> Result<(), InferenceError>;

    async fn reset_prefix_cache(&self) -> Result<(), InferenceError>;

    fn tp_size(&self) -> usize;
}

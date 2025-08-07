// src/inference/engine.rs

use async_trait::async_trait;
use crate::inference::types::*;

#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn generate(&self, input: InferenceEngineInput) -> Result<InferenceEngineOutput, InferenceError>;
    
    async fn wake_up(&self) -> Result<(), InferenceError>;
    
    async fn sleep(&self) -> Result<(), InferenceError>;
    
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
    
    async fn update_named_weight(&self, request: NamedWeightUpdateRequest) -> Result<(), InferenceError>;
    
    async fn teardown(&self) -> Result<(), InferenceError>;
    
    async fn reset_prefix_cache(&self) -> Result<(), InferenceError>;
}

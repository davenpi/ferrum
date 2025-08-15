// ferrum/src/train.rs
use crate::env::{Env, VecEnv};
use crate::learner::Learner;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TrainConfig {
    pub max_steps: Option<u64>,
    pub max_episodes: Option<u64>,
    pub coordinator_addr: Option<String>,

    pub inference_mode: InferenceMode,
}

#[derive(Debug, Clone)]
pub enum InferenceMode {
    Dedicated, // Inference and training run on different GPUs
    Colocated, // Inference and training run on the same GPUs (future work)
}

impl Default for TrainConfig {
    fn default() -> Self {
        Self {
            max_steps: Some(100_000),
            max_episodes: None,
            coordinator_addr: None,
            inference_mode: InferenceMode::Dedicated,
        }
    }
}

#[derive(Debug)]
pub struct TrainingStats {
    pub total_steps: u64,
    pub total_episodes: u64,
    pub training_time: Duration,
    pub final_version: u64,
}

// Start with a simple function, grow into trait later
pub async fn train<E, L>(
    #[allow(unused_variables)] env: VecEnv<E>,
    #[allow(unused_variables)] learner: &mut L,
    #[allow(unused_variables)] cfg: TrainConfig,
) -> Result<TrainingStats, Box<dyn std::error::Error>>
where
    E: Env + Send + 'static,
    L: Learner<E::Obs, E::Act> + Send + 'static,
{
    todo!("Implementation coming soon!")
}

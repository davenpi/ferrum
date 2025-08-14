use async_trait::async_trait;

use super::errors::ControlError;

/// Information about the current RL run (policy version, rollout config, endpoints).
pub struct RunInfo {
    pub current_version: PolicyVersion,
    pub rollout_quant: Option<QuantConfig>,
    pub inference_addrs: Vec<String>,
    pub learner_addr: String,
}

/// Coordinator trait for managing RL run state and parameter publishing.
#[async_trait]
pub trait Coordinator: Send + Sync {
    /// Fetch current run info (policy version, rollout config, endpoints).
    async fn get_run_info(&self) -> Result<RunInfo, ControlError>;
    /// Publish new policy parameters (version and checkpoint URI).
    async fn publish_params(&self, version: u64, uri: String);
    // optional: async fn heartbeat(&self, node_id: String, role: String) -> HeartbeatReply;
}

/// Precision types for model quantization.
pub enum Precision {
    FP32,
    BF16,
    FP16,
    FP8,
    INT8,
}

/// Version of the policy (checkpoint ID and precision).
pub struct PolicyVersion {
    pub id: u64,
    pub checkpoint_uri: String,    // base (high-precision) weights
    pub base_precision: Precision, // e.g., BF16
}

/// Configuration for quantization of rollout precision.
pub struct QuantConfig {
    pub precision: Precision, // rollout precision
    pub scheme: String,       // e.g., "awq", "fp8-e4m3"
    pub calib_uri: Option<String>,
}

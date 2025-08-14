use async_trait::async_trait;

pub struct RunInfo {
    pub current_version: PolicyVersion,
    pub rollout_quant: Option<QuantConfig>,
    pub inference_addrs: Vec<String>,
    pub learner_addr: String,
}

#[async_trait]
pub trait Coordinator: Send + Sync {
    async fn get_run_info(&self) -> RunInfo;
    async fn publish_params(&self, version: u64, uri: String);
    // optional: async fn heartbeat(&self, node_id: String, role: String) -> HeartbeatReply;
}

pub enum Precision {
    FP32,
    BF16,
    FP16,
    FP8,
    INT8,
}

pub struct PolicyVersion {
    pub id: u64,
    pub checkpoint_uri: String,    // base (high-precision) weights
    pub base_precision: Precision, // e.g., BF16
}

pub struct QuantConfig {
    pub precision: Precision, // rollout precision
    pub scheme: String,       // e.g., "awq", "fp8-e4m3"
    pub calib_uri: Option<String>,
}

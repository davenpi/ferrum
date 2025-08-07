
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// Equivalent to Python's MessageType = Dict[str, str]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,    // "user", "assistant", "system"
    pub content: String,
}

// Equivalent to Python's ConversationType = List[MessageType]
pub type Conversation = Vec<Message>;

// This is more type-safe than Python's Dict[str, Any]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingParams {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub stop: Option<Vec<String>>,
    // We can add more fields as needed, or use a HashMap for flexibility
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

// Rust equivalent of InferenceEngineInput
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceEngineInput {
    // Using Option<Vec<T>> like Python's Optional[List[T]]
    pub prompts: Option<Vec<Conversation>>,
    pub prompt_token_ids: Option<Vec<Vec<i32>>>,
    pub sampling_params: Option<SamplingParams>,
    // Using String for trajectory IDs (could be uuid::Uuid if we want type safety)
    pub trajectory_ids: Option<Vec<String>>,
}

// Better than Python strings - we can enumerate all possible stop reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StopReason {
    Stop,           // Normal completion
    Length,         // Hit max length
    Error,          // Some error occurred
    Timeout,        // Request timed out
    Other(String),  // Fallback for unknown reasons
}

// Rust equivalent of InferenceEngineOutput
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceEngineOutput {
    pub responses: Vec<String>,
    pub stop_reasons: Vec<StopReason>,  // Much better than Vec<String>!
}

// Rust equivalent of NamedWeightUpdateRequest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedWeightUpdateRequest {
    pub name: String,
    pub dtype: String,
    pub shape: Vec<usize>,  // usize is more idiomatic than i32 for sizes
    pub extras: Option<HashMap<String, serde_json::Value>>,
}

// Custom error type for our inference engine
#[derive(Debug, thiserror::Error)]
pub enum InferenceError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Engine communication error: {0}")]
    CommunicationError(String),
    #[error("Timeout after {seconds}s")]
    Timeout { seconds: u64 },
    #[error("Engine not available")]
    EngineUnavailable,
}


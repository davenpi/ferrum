use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step<O, A> {
    pub obs: O,
    pub act: A,
    pub rew: f32,
    pub done: bool,
    pub info: serde_json::Value, // Keep it simple for now
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryShard<O, A> {
    pub id: String,
    pub steps: Vec<Step<O, A>>,
    pub version: u64,
    pub rollout_probs: Option<Vec<f32>>, // For importance sampling
}

use super::engine::InferenceEngine;
use super::types::{
    InferenceEngineInput,
    InferenceEngineOutput,
    InferenceError,
};

pub struct InferenceEngineClient {
    engines: Vec<Box<dyn InferenceEngine>>,
}

impl InferenceEngineClient {
    pub fn new(engines: Vec<Box<dyn InferenceEngine>>) -> Self {
        Self { engines }
    }
}
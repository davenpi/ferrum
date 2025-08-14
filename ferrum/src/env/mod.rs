// ferrum/src/env/mod.rs
use crate::types::EnvError;

pub trait Env: Send {
    type Obs: Send + Clone + 'static;
    type Act: Send + Clone + 'static;
    type Info: Send + Clone + 'static;

    fn reset(&mut self) -> Result<Self::Obs, EnvError>;
    fn step(&mut self, act: Self::Act) -> Result<(Self::Obs, f32, bool, Self::Info), EnvError>;
    fn close(&mut self) -> Result<(), EnvError>;
}

// Simple test environment
pub struct DummyEnv {
    step_count: usize,
}

impl DummyEnv {
    pub fn new() -> Self {
        Self { step_count: 0 }
    }
}

impl Env for DummyEnv {
    type Obs = f32;
    type Act = f32;
    type Info = ();

    fn reset(&mut self) -> Result<Self::Obs, EnvError> {
        self.step_count = 0;
        Ok(0.0)
    }

    fn step(&mut self, _act: Self::Act) -> Result<(Self::Obs, f32, bool, Self::Info), EnvError> {
        self.step_count += 1;
        let done = self.step_count >= 10;
        Ok((self.step_count as f32, 1.0, done, ()))
    }
}

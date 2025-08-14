use super::{Env, EnvError};
use std::marker::PhantomData;

pub struct VecEnv<E> {
    envs: Vec<E>,
    _phantom: PhantomData<E>,
}

impl<E: Env> VecEnv<E> {
    pub fn new<F>(make_env: F, num_envs: usize) -> Self
    where
        F: Fn() -> E,
    {
        let envs = (0..num_envs).map(|_| make_env()).collect();
        Self {
            envs,
            _phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.envs.len()
    }

    pub fn reset(&mut self) -> Result<Vec<E::Obs>, EnvError> {
        self.envs.iter_mut().map(|env| env.reset()).collect()
    }

    pub fn step(
        &mut self,
        actions: Vec<E::Act>,
    ) -> Result<Vec<(E::Obs, f32, bool, E::Info)>, EnvError> {
        if actions.len() != self.envs.len() {
            return Err(EnvError::EnvError(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Action count doesn't match environment count",
            ))));
        }

        self.envs
            .iter_mut()
            .zip(actions)
            .map(|(env, action)| env.step(action))
            .collect()
    }

    pub fn close(&mut self) -> Result<(), EnvError> {
        for env in &mut self.envs {
            env.close()?;
        }
        Ok(())
    }
}

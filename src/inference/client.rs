use super::engine::InferenceEngine;
use super::types::{
    InferenceEngineInput, InferenceEngineOutput, InferenceError, Message, NamedWeightUpdateRequest,
    SamplingParams,
};
use async_trait::async_trait;
use futures::future::join_all;
use std::hash::{DefaultHasher, Hash, Hasher};

pub struct InferenceEngineClient {
    engines: Vec<Box<dyn InferenceEngine>>,
}

impl InferenceEngineClient {
    pub fn new(engines: Vec<Box<dyn InferenceEngine>>) -> Result<Self, InferenceError> {
        if engines.is_empty() {
            return Err(InferenceError::InvalidInput(
                "Must provide at least one engine".to_string(),
            ));
        }

        println!(
            "InferenceEngineClient initialized with {} engines.",
            engines.len()
        );
        Ok(Self { engines })
    }

    fn calculate_engine_index(&self, trajectory_id: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        trajectory_id.hash(&mut hasher);
        // Equivalent to Python's abs(hash(str(traj_id))) % len(self.engines)
        (hasher.finish() as usize) % self.engines.len()
    }

    async fn generate_with_trajectory_routing(
        &self,
        input: InferenceEngineInput,
    ) -> Result<InferenceEngineOutput, InferenceError> {
        // TODO: Implement trajectory-based routing (complex batching with ordering)
        // This is used for multi-turn agent loops with trajectory_ids
        // Need to:
        // 1. Group prompts by engine using hash(trajectory_id) % num_engines
        // 2. Create tasks for each engine group
        // 3. Execute tasks in parallel
        // 4. Reconstruct results in original order using indices
        println!("generate_with_trajectory_routing {:?}", input);
        todo!("Implement trajectory routing - come back after generate_batched works")
    }

    async fn generate_batched(
        &self,
        input: InferenceEngineInput,
    ) -> Result<InferenceEngineOutput, InferenceError> {
        // Handle the two different input types separately with pattern matching
        match (&input.prompts, &input.prompt_token_ids) {
            (Some(prompts), None) => {
                // We have conversation prompts
                self.generate_batched_prompts(prompts, &input.sampling_params)
                    .await
            }
            (None, Some(token_ids)) => {
                // We have token IDs
                self.generate_batched_token_ids(token_ids, &input.sampling_params)
                    .await
            }
            _ => {
                // This should never happen due to validation in generate()
                Err(InferenceError::InvalidInput(
                    "Invalid input state in generate_batched".to_string(),
                ))
            }
        }
    }

    async fn generate_batched_prompts(
        &self,
        prompts: &Vec<Vec<Message>>,
        sampling_params: &Option<SamplingParams>,
    ) -> Result<InferenceEngineOutput, InferenceError> {
        let num_engines = self.engines.len();
        let batch_size = prompts.len().div_ceil(num_engines);

        let mut tasks = Vec::new();

        for engine_idx in 0..num_engines {
            let start_idx = engine_idx * batch_size;
            let end_idx = ((engine_idx + 1) * batch_size).min(prompts.len());

            if start_idx >= prompts.len() {
                continue;
            }

            let batch_items = prompts[start_idx..end_idx].to_vec();
            let engine_input = InferenceEngineInput {
                prompts: Some(batch_items),
                prompt_token_ids: None,
                sampling_params: sampling_params.clone(),
                trajectory_ids: None,
            };

            tasks.push(self.engines[engine_idx].generate(engine_input));
        }

        // Execute all tasks and flatten results
        let results = join_all(tasks).await;
        let mut responses = Vec::new();
        let mut stop_reasons = Vec::new();

        for result in results {
            let result = result?;
            responses.extend(result.responses);
            stop_reasons.extend(result.stop_reasons);
        }

        Ok(InferenceEngineOutput {
            responses,
            stop_reasons,
        })
    }

    async fn generate_batched_token_ids(
        &self,
        token_ids: &Vec<Vec<i32>>,
        sampling_params: &Option<SamplingParams>,
    ) -> Result<InferenceEngineOutput, InferenceError> {
        let num_engines = self.engines.len();
        let batch_size = (token_ids.len() + num_engines - 1) / num_engines; // Ceiling division

        let mut tasks = Vec::new();

        for engine_idx in 0..num_engines {
            let start_idx = engine_idx * batch_size;
            let end_idx = ((engine_idx + 1) * batch_size).min(token_ids.len());

            if start_idx >= token_ids.len() {
                continue;
            }

            let batch_items = token_ids[start_idx..end_idx].to_vec();
            let engine_input = InferenceEngineInput {
                prompts: None,
                prompt_token_ids: Some(batch_items),
                sampling_params: sampling_params.clone(),
                trajectory_ids: None,
            };

            tasks.push(self.engines[engine_idx].generate(engine_input));
        }

        // Execute all tasks and flatten results
        let results = join_all(tasks).await;
        let mut responses = Vec::new();
        let mut stop_reasons = Vec::new();

        for result in results {
            let result = result?;
            responses.extend(result.responses);
            stop_reasons.extend(result.stop_reasons);
        }

        Ok(InferenceEngineOutput {
            responses,
            stop_reasons,
        })
    }
}

#[async_trait]
impl InferenceEngine for InferenceEngineClient {
    fn tp_size(&self) -> usize {
        // Sum up the tp_size of all engines
        self.engines.iter().map(|engine| engine.tp_size()).sum()
    }

    async fn generate(
        &self,
        input: InferenceEngineInput,
    ) -> Result<InferenceEngineOutput, InferenceError> {
        // Step 1: Validate input
        match (&input.prompts, &input.prompt_token_ids) {
            (None, None) | (Some(_), Some(_)) => {
                return Err(InferenceError::InvalidInput(
                    "Either prompts or prompt_token_ids must be provided, but not both".to_string(),
                ));
            }
            _ => {}
        }

        // Step 2: Route based on whether we have trajectory IDs
        if input.trajectory_ids.is_some() {
            self.generate_with_trajectory_routing(input).await
        } else {
            self.generate_batched(input).await
        }
    }

    async fn wake_up(&self, tags: Option<Vec<String>>) -> Result<(), InferenceError> {
        let tasks: Vec<_> = self
            .engines
            .iter()
            .map(|engine| engine.wake_up(tags.clone()))
            .collect();

        let results = join_all(tasks).await;

        // Check if any failed
        for result in results {
            result?; // Propagate any errors
        }

        Ok(())
    }

    async fn sleep(&self, level: Option<i32>) -> Result<(), InferenceError> {
        let tasks: Vec<_> = self
            .engines
            .iter()
            .map(|engine| engine.sleep(level))
            .collect();

        let results = join_all(tasks).await;

        for result in results {
            result?;
        }

        Ok(())
    }

    // TODO: Implement the remaining methods...
    async fn init_weight_update_communicator(
        &self,
        master_addr: String,
        master_port: u16,
        rank_offset: usize,
        world_size: usize,
        group_name: String,
        backend: String,
        override_existing: bool,
    ) -> Result<(), InferenceError> {
        let mut tasks = Vec::new();
        let mut rank_offset_count = rank_offset;

        for engine in &self.engines {
            let tp_size = engine.tp_size();

            tasks.push(engine.init_weight_update_communicator(
                master_addr.clone(),
                master_port,
                rank_offset_count,
                world_size,
                group_name.clone(),
                backend.clone(),
                override_existing,
            ));

            rank_offset_count += tp_size;
        }

        let results = join_all(tasks).await;

        for result in results {
            result?;
        }

        Ok(())
    }

    async fn update_named_weight(
        &self,
        request: NamedWeightUpdateRequest,
    ) -> Result<(), InferenceError> {
        let tasks: Vec<_> = self
            .engines
            .iter()
            .map(|engine| engine.update_named_weight(request.clone()))
            .collect();

        let results = join_all(tasks).await;

        for result in results {
            result?;
        }

        Ok(())
    }

    async fn teardown(&self) -> Result<(), InferenceError> {
        let tasks: Vec<_> = self
            .engines
            .iter()
            .map(|engine| engine.teardown())
            .collect();

        let results = join_all(tasks).await;

        for result in results {
            result?;
        }

        Ok(())
    }

    async fn reset_prefix_cache(&self) -> Result<(), InferenceError> {
        let tasks: Vec<_> = self
            .engines
            .iter()
            .map(|engine| engine.reset_prefix_cache())
            .collect();

        let results = join_all(tasks).await;

        for result in results {
            result?;
        }

        Ok(())
    }
}

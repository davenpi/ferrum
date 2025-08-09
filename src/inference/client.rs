use super::engine::InferenceEngine;
use super::types::{
    InferenceEngineInput, InferenceEngineOutput, InferenceError, NamedWeightUpdateRequest,
};
use async_trait::async_trait;
use futures::future::join_all;
use std::collections::HashMap;
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
        let trajectory_ids = input.trajectory_ids.as_ref().ok_or_else(|| {
            InferenceError::InvalidInput(
                "trajectory_ids required for trajectory routing".to_string(),
            )
        })?;

        // 1. Validate and determine input type
        let (input_length, is_prompts) = self.validate_and_classify_input(&input)?;
        self.validate_trajectory_length(trajectory_ids, input_length)?;

        // 2. Group inputs by engine
        let engine_groups = self.group_by_engine(trajectory_ids);

        // 3. Execute in parallel
        let (engine_outputs, indices_lists) = self
            .execute_engine_groups(engine_groups, &input, is_prompts)
            .await?;

        // 4. Reconstruct results in original order
        let output = self.reconstruct_output(engine_outputs, indices_lists, input_length)?;

        Ok(output)
    }

    async fn generate_batched(
        &self,
        input: InferenceEngineInput,
    ) -> Result<InferenceEngineOutput, InferenceError> {
        // Handle the two different input types separately with pattern matching
        match (&input.prompts, &input.prompt_token_ids) {
            (Some(prompts), None) => {
                // We have conversation prompts
                self.generate_batched_generic(prompts, |batch_items| InferenceEngineInput {
                    prompts: Some(batch_items),
                    prompt_token_ids: None,
                    sampling_params: input.sampling_params.clone(),
                    trajectory_ids: None,
                })
                .await
            }
            (None, Some(token_ids)) => {
                // We have token IDs
                self.generate_batched_generic(token_ids, |batch_items| InferenceEngineInput {
                    prompts: None,
                    prompt_token_ids: Some(batch_items),
                    sampling_params: input.sampling_params.clone(),
                    trajectory_ids: None,
                })
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

    async fn generate_batched_generic<T: Clone>(
        &self,
        items: &[T],
        create_engine_input: impl Fn(Vec<T>) -> InferenceEngineInput,
    ) -> Result<InferenceEngineOutput, InferenceError> {
        let num_engines = self.engines.len();
        let batch_size = items.len().div_ceil(num_engines);

        let mut tasks = Vec::new();

        for engine_idx in 0..num_engines {
            let start_idx = engine_idx * batch_size;
            let end_idx = ((engine_idx + 1) * batch_size).min(items.len());

            if start_idx >= items.len() {
                continue;
            }

            let batch_items = items[start_idx..end_idx].to_vec();
            let engine_input = create_engine_input(batch_items);

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

// Helper methods
impl InferenceEngineClient {
    fn validate_and_classify_input(
        &self,
        input: &InferenceEngineInput,
    ) -> Result<(usize, bool), InferenceError> {
        match (&input.prompts, &input.prompt_token_ids) {
            (Some(prompts), None) => Ok((prompts.len(), true)),
            (None, Some(tokens)) => Ok((tokens.len(), false)),
            _ => Err(InferenceError::InvalidInput(
                "Invalid input state in trajectory routing".to_string(),
            )),
        }
    }

    fn validate_trajectory_length(
        &self,
        trajectory_ids: &[String],
        input_length: usize,
    ) -> Result<(), InferenceError> {
        if trajectory_ids.len() != input_length {
            return Err(InferenceError::InvalidInput(format!(
                "trajectory_ids length ({}) must match input length ({})",
                trajectory_ids.len(),
                input_length
            )));
        }
        Ok(())
    }

    fn group_by_engine(&self, trajectory_ids: &[String]) -> HashMap<usize, Vec<usize>> {
        let mut engine_groups = HashMap::new();

        for (original_idx, trajectory_id) in trajectory_ids.iter().enumerate() {
            let engine_idx = self.calculate_engine_index(trajectory_id);
            engine_groups
                .entry(engine_idx)
                .or_insert_with(Vec::new)
                .push(original_idx);
        }

        engine_groups
    }

    async fn execute_engine_groups(
        &self,
        engine_groups: HashMap<usize, Vec<usize>>,
        input: &InferenceEngineInput,
        is_prompts: bool,
    ) -> Result<(Vec<InferenceEngineOutput>, Vec<Vec<usize>>), InferenceError> {
        let mut tasks = Vec::new();
        let mut indices_lists = Vec::new();

        for (engine_idx, indices) in engine_groups {
            let engine_input = self.build_engine_input(&indices, input, is_prompts)?;
            tasks.push(self.engines[engine_idx].generate(engine_input));
            indices_lists.push(indices);
        }

        // Execute all tasks in parallel
        let results = join_all(tasks).await;

        // Check for errors and collect outputs
        let mut engine_outputs = Vec::new();
        for result in results {
            engine_outputs.push(result?);
        }

        Ok((engine_outputs, indices_lists))
    }

    fn build_engine_input(
        &self,
        indices: &[usize],
        input: &InferenceEngineInput,
        is_prompts: bool,
    ) -> Result<InferenceEngineInput, InferenceError> {
        if is_prompts {
            let prompts = input.prompts.as_ref().unwrap();
            let group_prompts: Vec<_> = indices.iter().map(|&idx| prompts[idx].clone()).collect();

            Ok(InferenceEngineInput {
                prompts: Some(group_prompts),
                prompt_token_ids: None,
                sampling_params: input.sampling_params.clone(),
                trajectory_ids: None,
            })
        } else {
            let token_ids = input.prompt_token_ids.as_ref().unwrap();
            let group_token_ids: Vec<_> =
                indices.iter().map(|&idx| token_ids[idx].clone()).collect();

            Ok(InferenceEngineInput {
                prompts: None,
                prompt_token_ids: Some(group_token_ids),
                sampling_params: input.sampling_params.clone(),
                trajectory_ids: None,
            })
        }
    }

    fn reconstruct_output(
        &self,
        engine_outputs: Vec<InferenceEngineOutput>,
        indices_lists: Vec<Vec<usize>>,
        input_length: usize,
    ) -> Result<InferenceEngineOutput, InferenceError> {
        let mut responses = vec![String::new(); input_length];
        let mut stop_reasons =
            vec![crate::inference::types::StopReason::Other("unset".to_string()); input_length];

        for (indices, output) in indices_lists.iter().zip(engine_outputs.iter()) {
            for (local_idx, &original_idx) in indices.iter().enumerate() {
                responses[original_idx] = output.responses[local_idx].clone();
                stop_reasons[original_idx] = output.stop_reasons[local_idx].clone();
            }
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

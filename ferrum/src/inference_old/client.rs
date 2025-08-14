use super::engine::InferenceEngine;
use super::types::{
    InferenceEngineInput, InferenceEngineOutput, InferenceError, Message, NamedWeightUpdateRequest,
    SamplingParams,
};
use async_trait::async_trait;
use futures::future::join_all;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, Clone)]
struct TrajectoryItem {
    trajectory_id: String,
    original_index: usize,
    data: TrajectoryData,
}

#[derive(Debug, Clone)]
enum TrajectoryData {
    Prompts(Vec<Message>),
    Tokens(Vec<i32>),
}

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
        (hasher.finish() as usize) % self.engines.len()
    }

    async fn generate_with_trajectory_routing(
        &self,
        input: InferenceEngineInput,
    ) -> Result<InferenceEngineOutput, InferenceError> {
        // 1. Convert input to trajectory items
        let trajectory_items = self.create_trajectory_items(&input)?;

        // 2. Group by engine
        let engine_groups = self.group_trajectories_by_engine(trajectory_items);

        // 3. Execute engines
        let engine_results = self
            .execute_trajectory_groups(engine_groups, &input.sampling_params)
            .await?;

        // 4. Reconstruct
        let output = self.reconstruct_by_original_index(engine_results, &input)?;

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
    fn create_trajectory_items(
        &self,
        input: &InferenceEngineInput,
    ) -> Result<Vec<TrajectoryItem>, InferenceError> {
        let trajectory_ids = input
            .trajectory_ids
            .as_ref()
            .ok_or_else(|| InferenceError::InvalidInput("trajectory_ids required".to_string()))?;

        let mut traj_items = Vec::new();

        if let Some(prompts) = &input.prompts {
            if prompts.len() != trajectory_ids.len() {
                return Err(InferenceError::InvalidInput(format!(
                    "prompts length ({}) must match trajectory_ids length ({})",
                    prompts.len(),
                    trajectory_ids.len()
                )));
            }

            for (idx, (prompt, traj_id)) in prompts.iter().zip(trajectory_ids.iter()).enumerate() {
                traj_items.push(TrajectoryItem {
                    trajectory_id: traj_id.clone(),
                    original_index: idx,
                    data: TrajectoryData::Prompts(prompt.clone()),
                });
            }
        } else if let Some(tokens) = &input.prompt_token_ids {
            if tokens.len() != trajectory_ids.len() {
                return Err(InferenceError::InvalidInput(format!(
                    "tokens length ({}) must match trajectory_ids length ({})",
                    tokens.len(),
                    trajectory_ids.len()
                )));
            }

            for (idx, (token, traj_id)) in tokens.iter().zip(trajectory_ids.iter()).enumerate() {
                traj_items.push(TrajectoryItem {
                    trajectory_id: traj_id.clone(),
                    original_index: idx,
                    data: TrajectoryData::Tokens(token.clone()),
                });
            }
        } else {
            return Err(InferenceError::InvalidInput(
                "Either prompts or prompt_token_ids must be provided".to_string(),
            ));
        }

        Ok(traj_items)
    }

    fn group_trajectories_by_engine(
        &self,
        traj_items: Vec<TrajectoryItem>,
    ) -> HashMap<usize, Vec<TrajectoryItem>> {
        let mut engine_groups = HashMap::new();

        for traj_item in traj_items {
            let engine_idx = self.calculate_engine_index(&traj_item.trajectory_id);
            engine_groups
                .entry(engine_idx)
                .or_insert_with(Vec::new)
                .push(traj_item);
        }

        engine_groups
    }

    async fn execute_trajectory_groups(
        &self,
        engine_groups: HashMap<usize, Vec<TrajectoryItem>>,
        sampling_params: &Option<SamplingParams>,
    ) -> Result<HashMap<usize, (Vec<TrajectoryItem>, InferenceEngineOutput)>, InferenceError> {
        let mut tasks = Vec::new();
        let mut engine_indices = Vec::new();
        let mut traj_item_lists = Vec::new();

        for (engine_idx, traj_items) in engine_groups {
            let engine_input =
                self.build_engine_input_from_traj_items(&traj_items, sampling_params)?;

            tasks.push(self.engines[engine_idx].generate(engine_input));
            engine_indices.push(engine_idx);
            traj_item_lists.push(traj_items);
        }

        let results = join_all(tasks).await;

        // Rebuild the semantic mapping
        let mut engine_results = HashMap::new();
        for ((engine_idx, traj_items), result) in engine_indices
            .into_iter()
            .zip(traj_item_lists.into_iter())
            .zip(results.into_iter())
        {
            engine_results.insert(engine_idx, (traj_items, result?));
        }

        Ok(engine_results)
    }

    fn build_engine_input_from_traj_items(
        &self,
        traj_items: &[TrajectoryItem],
        sampling_params: &Option<SamplingParams>,
    ) -> Result<InferenceEngineInput, InferenceError> {
        if traj_items.is_empty() {
            return Err(InferenceError::InvalidInput("Empty batch".to_string()));
        }

        // Check if all batches are the same type
        let is_prompts = matches!(traj_items[0].data, TrajectoryData::Prompts(_));

        if is_prompts {
            let mut prompts = Vec::new();
            for traj_item in traj_items {
                if let TrajectoryData::Prompts(prompt) = &traj_item.data {
                    prompts.push(prompt.clone());
                } else {
                    return Err(InferenceError::InvalidInput(
                        "Mixed prompt and token data in same batch".to_string(),
                    ));
                }
            }

            Ok(InferenceEngineInput {
                prompts: Some(prompts),
                prompt_token_ids: None,
                sampling_params: sampling_params.clone(),
                trajectory_ids: None,
            })
        } else {
            let mut tokens = Vec::new();
            for traj_item in traj_items {
                if let TrajectoryData::Tokens(token) = &traj_item.data {
                    tokens.push(token.clone());
                } else {
                    return Err(InferenceError::InvalidInput(
                        "Mixed prompt and token data in same batch".to_string(),
                    ));
                }
            }

            Ok(InferenceEngineInput {
                prompts: None,
                prompt_token_ids: Some(tokens),
                sampling_params: sampling_params.clone(),
                trajectory_ids: None,
            })
        }
    }

    fn reconstruct_by_original_index(
        &self,
        engine_results: HashMap<usize, (Vec<TrajectoryItem>, InferenceEngineOutput)>,
        original_input: &InferenceEngineInput,
    ) -> Result<InferenceEngineOutput, InferenceError> {
        // Figure out total length
        let total_length = if let Some(prompts) = &original_input.prompts {
            prompts.len()
        } else if let Some(tokens) = &original_input.prompt_token_ids {
            tokens.len()
        } else {
            return Err(InferenceError::InvalidInput("No input data".to_string()));
        };

        let mut responses = vec![String::new(); total_length];
        let mut stop_reasons =
            vec![crate::inference_old::types::StopReason::Other("unset".to_string()); total_length];

        // Now we can iterate semantically over the engine results
        for (_, (traj_items, output)) in engine_results {
            // Place each result back in its original position
            for (traj_item, (response, stop_reason)) in traj_items
                .iter()
                .zip(output.responses.iter().zip(output.stop_reasons.iter()))
            {
                responses[traj_item.original_index] = response.clone();
                stop_reasons[traj_item.original_index] = stop_reason.clone();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_with_trajectory_routing_three_prompts() {
        // Arrange
        let client = InferenceEngineClient {
            engines: vec![
                Box::new(MockEngine::new("Engine0")),
                Box::new(MockEngine::new("Engine1")),
                Box::new(MockEngine::new("Engine2")),
            ],
        };

        // Create input with 3 prompts and trajectory IDs
        let input = InferenceEngineInput {
            prompts: Some(vec![
                vec![Message {
                    role: "user".to_string(),
                    content: "Prompt 1".to_string(),
                }],
                vec![Message {
                    role: "user".to_string(),
                    content: "Prompt 2".to_string(),
                }],
                vec![Message {
                    role: "user".to_string(),
                    content: "Prompt 3".to_string(),
                }],
            ]),
            prompt_token_ids: None,
            sampling_params: Some(SamplingParams {
                temperature: Some(0.7),
                max_tokens: Some(100),
                top_p: Some(0.9),
                top_k: Some(40),
                stop: None,
                extra: None,
            }),
            trajectory_ids: Some(vec![
                "traj_001".to_string(),
                "traj_002".to_string(),
                "traj_003".to_string(),
            ]),
        };

        // Act
        let result = client.generate_with_trajectory_routing(input).await;

        // Assert
        assert!(result.is_ok(), "Should succeed with valid input");
        let output = result.unwrap();

        // Should get back 3 responses in original order
        assert_eq!(output.responses.len(), 3, "Should have 3 responses");
        assert_eq!(output.stop_reasons.len(), 3, "Should have 3 stop reasons");

        // Responses should be in original order (Engine0 responds with "Response from Engine0", etc.)
        assert!(
            output.responses[0].contains("Engine"),
            "First response should be from an engine"
        );
        assert!(
            output.responses[1].contains("Engine"),
            "Second response should be from an engine"
        );
        assert!(
            output.responses[2].contains("Engine"),
            "Third response should be from an engine"
        );
    }

    struct MockEngine {
        name: String,
    }

    impl MockEngine {
        fn new(name: &str) -> Self {
            MockEngine {
                name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl InferenceEngine for MockEngine {
        fn tp_size(&self) -> usize {
            1
        }

        async fn generate(
            &self,
            input: InferenceEngineInput,
        ) -> Result<InferenceEngineOutput, InferenceError> {
            // Return responses that identify which engine processed them
            let num_prompts = if let Some(prompts) = &input.prompts {
                prompts.len()
            } else if let Some(tokens) = &input.prompt_token_ids {
                tokens.len()
            } else {
                return Err(InferenceError::InvalidInput(
                    "No input provided".to_string(),
                ));
            };

            let responses: Vec<String> = (0..num_prompts)
                .map(|i| format!("Response from {} for item {}", self.name, i))
                .collect();

            let stop_reasons = vec![crate::inference_old::types::StopReason::Stop; num_prompts];

            Ok(InferenceEngineOutput {
                responses,
                stop_reasons,
            })
        }

        async fn wake_up(&self, _tags: Option<Vec<String>>) -> Result<(), InferenceError> {
            Ok(())
        }

        async fn sleep(&self, _level: Option<i32>) -> Result<(), InferenceError> {
            Ok(())
        }

        async fn init_weight_update_communicator(
            &self,
            _master_addr: String,
            _master_port: u16,
            _rank_offset: usize,
            _world_size: usize,
            _group_name: String,
            _backend: String,
            _override_existing: bool,
        ) -> Result<(), InferenceError> {
            Ok(())
        }

        async fn update_named_weight(
            &self,
            _request: NamedWeightUpdateRequest,
        ) -> Result<(), InferenceError> {
            Ok(())
        }

        async fn teardown(&self) -> Result<(), InferenceError> {
            Ok(())
        }

        async fn reset_prefix_cache(&self) -> Result<(), InferenceError> {
            Ok(())
        }
    }
}

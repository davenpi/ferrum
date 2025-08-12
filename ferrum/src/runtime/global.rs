use crate::runtime::{LocalScheduler, Scheduler, Task, TaskHandle};
use std::sync::OnceLock;

// Use concrete type instead of dyn Scheduler
static GLOBAL_SCHEDULER: OnceLock<LocalScheduler> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub workers: Option<usize>,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self { workers: None }
    }
}

pub fn init() -> Result<(), String> {
    init_with_config(SchedulerConfig::default())
}

pub fn init_with_config(_config: SchedulerConfig) -> Result<(), String> {
    // For now, just ensure scheduler gets initialized
    // Later we can use config when creating different scheduler types
    let _ = get_or_init_scheduler();
    Ok(())
}

pub fn get_or_init_scheduler() -> &'static LocalScheduler {
    GLOBAL_SCHEDULER.get_or_init(|| LocalScheduler::new())
}

// This is what the macro will call
pub fn submit<T>(task: T) -> TaskHandle<T::Output>
where
    T: Task + 'static,
{
    get_or_init_scheduler().submit(task)
}

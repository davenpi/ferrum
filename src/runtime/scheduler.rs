use crate::runtime::task::DistributedTask;
use async_trait::async_trait;

#[async_trait]
pub trait Scheduler: Send + Sync {
    async fn submit<T>(&self, task: T) -> Result<T::Output, String>
    where
        T: DistributedTask + 'static;
}

// Simple local scheduler to start with
pub struct LocalScheduler {
    // We'll add state here as we build it out
}

impl LocalScheduler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Scheduler for LocalScheduler {
    async fn submit<T>(&self, task: T) -> Result<T::Output, String>
    where
        T: DistributedTask + 'static,
    {
        // For now, just execute locally
        Ok(task.call().await)
    }
}

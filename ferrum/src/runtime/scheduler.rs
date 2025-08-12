use crate::runtime::handle::TaskHandle;
use crate::runtime::task::Task;
use async_trait::async_trait;
use tokio::sync::oneshot;
use uuid::Uuid;

#[async_trait]
pub trait Scheduler: Send + Sync {
    fn submit<T>(&self, task: T) -> TaskHandle<T::Output>
    where
        T: Task + 'static;
}

// Simple local scheduler
pub struct LocalScheduler {}

impl LocalScheduler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Scheduler for LocalScheduler {
    fn submit<T>(&self, task: T) -> TaskHandle<T::Output>
    where
        T: Task + 'static,
    {
        let task_id = Uuid::new_v4();
        let (sender, receiver) = oneshot::channel();

        // Spawn the task execution
        tokio::spawn(async move {
            let result = task.call().await;
            let _ = sender.send(Ok(result));
        });

        TaskHandle::new(task_id, receiver)
    }
}

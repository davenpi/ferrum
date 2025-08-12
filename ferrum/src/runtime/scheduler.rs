use crate::runtime::{Error, handle::TaskHandle, result_source::LocalResultSource, task::Task};
use async_trait::async_trait;
use std::future::Future;
use tokio::sync::oneshot;
use uuid::Uuid;

#[async_trait]
pub trait Scheduler: Send + Sync {
    type Handle<T>: Future<Output = Result<T, Error>> + Send
    where
        T: Send + 'static;

    fn submit<T>(&self, task: T) -> Self::Handle<T::Output>
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
    type Handle<T>
        = TaskHandle<T, LocalResultSource<T>>
    where
        T: Send + 'static;

    fn submit<T>(&self, task: T) -> Self::Handle<T::Output>
    where
        T: Task + 'static,
    {
        let task_id = Uuid::new_v4();
        let (sender, receiver) = oneshot::channel::<Result<T::Output, Error>>();

        // Spawn the task execution
        tokio::spawn(async move {
            let result = task.call().await;
            let _ = sender.send(Ok(result));
        });

        TaskHandle::new(task_id, LocalResultSource::new(receiver))
    }
}

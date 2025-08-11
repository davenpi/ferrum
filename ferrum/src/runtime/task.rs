use serde::{Deserialize, Serialize};
use std::future::Future;
use uuid::Uuid;

pub type TaskId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult<T> {
    pub task_id: TaskId,
    pub result: Result<T, String>,
}

// Renamed from DistributedTask to Task
pub trait Task: Send + Sync {
    type Output: Send + Sync + Serialize + for<'de> Deserialize<'de>;
    type Future: Future<Output = Self::Output> + Send;
    
    fn call(self) -> Self::Future;
}

// This is what the macro will generate for each function
pub struct TaskWrapper<F, Fut, T> 
where
    F: FnOnce() -> Fut + Send + Sync,
    Fut: Future<Output = T> + Send,
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
{
    pub func: F,
}

impl<F, Fut, T> TaskWrapper<F, Fut, T>
where
    F: FnOnce() -> Fut + Send + Sync,
    Fut: Future<Output = T> + Send,
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F, Fut, T> Task for TaskWrapper<F, Fut, T>
where
    F: FnOnce() -> Fut + Send + Sync,
    Fut: Future<Output = T> + Send,
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
{
    type Output = T;
    type Future = Fut;
    
    fn call(self) -> Self::Future {
        (self.func)()
    }
}

// Temporary macro until we have the proc macro working
#[macro_export]
macro_rules! ferrum_task {
    ($func:expr) => {{
        $crate::runtime::task::TaskWrapper::new($func)
    }};
}

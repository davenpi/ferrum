use serde::{Deserialize, Serialize};
use std::future::Future;

// task.rs - clean, no IDs
pub trait Task: Send + Sync {
    type Output: Send + Sync + Serialize + for<'de> Deserialize<'de>;
    type Future: Future<Output = Self::Output> + Send;
    fn call(self) -> Self::Future;
}

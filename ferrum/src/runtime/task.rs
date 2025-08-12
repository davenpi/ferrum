use std::future::Future;
use std::pin::Pin;

// A task is something we can move to a worker and await a result from.
pub trait Task: Send + 'static {
    type Output: Send + 'static;
    fn call(self) -> Pin<Box<dyn Future<Output = Self::Output> + Send>>;
}

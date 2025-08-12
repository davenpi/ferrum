use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;

// Abstract trait for result retrieval
pub trait ResultSource<T>: Future<Output = Result<T, String>> + Send + Unpin
where
    T: Send + Sync,
{
    // Future methods for cancellation, status checking, etc.
}

// Local implementation using oneshot channels
#[derive(Debug)]
pub struct LocalResultSource<T>
where
    T: Send + Sync,
{
    receiver: oneshot::Receiver<Result<T, String>>,
}

impl<T> LocalResultSource<T>
where
    T: Send + Sync,
{
    pub(crate) fn new(receiver: oneshot::Receiver<Result<T, String>>) -> Self {
        Self { receiver }
    }
}

impl<T> Future for LocalResultSource<T>
where
    T: Send + Sync,
{
    type Output = Result<T, String>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.receiver)
            .poll(cx)
            .map(|result| match result {
                Ok(task_result) => task_result,
                Err(_) => Err("Task was cancelled or scheduler dropped".to_string()),
            })
    }
}

impl<T> ResultSource<T> for LocalResultSource<T> where T: Send + Sync {}

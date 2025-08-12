use crate::runtime::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;

// Abstract trait for result retrieval
pub trait ResultSource<T>: Future<Output = Result<T, Error>> + Send + Unpin
where
    T: Send + 'static,
{
    // Future methods for cancellation, status checking, etc.
}

// Local implementation using oneshot channels
#[derive(Debug)]
pub struct LocalResultSource<T>
where
    T: Send + 'static,
{
    receiver: oneshot::Receiver<Result<T, Error>>,
}

impl<T> LocalResultSource<T>
where
    T: Send + 'static,
{
    pub(crate) fn new(receiver: oneshot::Receiver<Result<T, Error>>) -> Self {
        Self { receiver }
    }
}

impl<T> Future for LocalResultSource<T>
where
    T: Send + 'static,
{
    type Output = Result<T, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.receiver)
            .poll(cx)
            .map(|result| match result {
                Ok(task_result) => task_result,
                Err(_) => Err(Error::Canceled),
            })
    }
}

impl<T> ResultSource<T> for LocalResultSource<T> where T: Send + 'static {}

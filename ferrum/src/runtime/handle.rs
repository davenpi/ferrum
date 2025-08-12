use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;
use uuid::Uuid;

pub struct TaskHandle<T> {
    id: Uuid,
    receiver: oneshot::Receiver<Result<T, String>>,
}

impl<T> TaskHandle<T> {
    pub(crate) fn new(id: Uuid, receiver: oneshot::Receiver<Result<T, String>>) -> Self {
        Self { id, receiver }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub async fn cancel(self) -> Result<(), String> {
        // TODO: Implement cancellation logic
        // For now, just drop the receiver
        drop(self.receiver);
        Ok(())
    }
}

impl<T> Future for TaskHandle<T> {
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

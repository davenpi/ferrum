use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use crate::runtime::error::Error;
use crate::runtime::result_source::{LocalResultSource, ResultSource};
use pin_project_lite::pin_project;
use uuid::Uuid;

pin_project! {
    /// A handle to a submitted task, which can be awaited to get the result.
    ///
    /// `TaskHandle` is a future that represents the completion of a task scheduled
    /// on a runtime. When awaited, it yields the output of the task.
    ///
    /// This struct is a wrapper around a `ResultSource`, which is the actual
    /// future that waits for the task's result to become available.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The output type of the task. This is the value that the future will
    ///        resolve to upon completion.
    /// * `S`: The specific implementation of `ResultSource` used to retrieve
    ///        the result. It defaults to `LocalResultSource<T>` for the
    ///        local scheduler.
    pub struct TaskHandle<T, S = LocalResultSource<T>> {
        id: Uuid,
        #[pin]
        source: S,
        _phantom: PhantomData<T>,
    }
}

// Manual Debug implementation - works regardless of whether T implements Debug
impl<T, S> fmt::Debug for TaskHandle<T, S>
where
    T: Send + 'static,
    S: ResultSource<T> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskHandle")
            .field("id", &self.id)
            .field("source", &self.source)
            .field("result_type", &std::any::type_name::<T>())
            .finish()
    }
}

impl<T, S> TaskHandle<T, S>
where
    T: Send + 'static,
    S: ResultSource<T>,
{
    pub(crate) fn new(id: Uuid, source: S) -> Self {
        Self {
            id,
            source,
            _phantom: PhantomData,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub async fn cancel(self) -> Result<(), Error> {
        drop(self.source);
        Ok(())
    }

    /// Transform the result of this task handle
    pub fn map<U, F>(self, f: F) -> impl Future<Output = Result<U, Error>> + Send
    where
        U: Send + 'static,
        F: FnOnce(T) -> U + Send + 'static,
    {
        async move { self.await.map(f) }
    }

    /// Add a timeout to this task handle
    pub fn timeout(self, dur: Duration) -> impl Future<Output = Result<T, Error>> + Send {
        async move {
            match tokio::time::timeout(dur, self).await {
                Ok(r) => r,
                Err(_) => Err(Error::Timeout),
            }
        }
    }
}

impl<T, S> Future for TaskHandle<T, S>
where
    T: Send + 'static,
    S: ResultSource<T>,
{
    type Output = Result<T, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safe pin projection using pin-project-lite
        self.project().source.poll(cx)
    }
}

// Type alias for the common local case
pub type LocalTaskHandle<T> = TaskHandle<T, LocalResultSource<T>>;

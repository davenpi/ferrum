use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::runtime::result_source::{LocalResultSource, ResultSource};
use uuid::Uuid;

// TaskHandle is now generic over the result source
pub struct TaskHandle<T, S = LocalResultSource<T>>
where
    T: Send + Sync,
    S: ResultSource<T>,
{
    id: Uuid,
    source: S,
    _phantom: PhantomData<T>,
}

// Manual Debug implementation - works regardless of whether T implements Debug
impl<T, S> fmt::Debug for TaskHandle<T, S>
where
    T: Send + Sync,
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
    T: Send + Sync,
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

    pub async fn cancel(self) -> Result<(), String> {
        // TODO: Implement cancellation logic based on source type
        // For now, just drop the source
        drop(self.source);
        Ok(())
    }
}

impl<T, S> Future for TaskHandle<T, S>
where
    T: Send + Sync,
    S: ResultSource<T>,
{
    type Output = Result<T, String>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: We never move out of the pinned field
        let source = unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.source) };
        source.poll(cx)
    }
}

// Type alias for the common local case
pub type LocalTaskHandle<T> = TaskHandle<T, LocalResultSource<T>>;

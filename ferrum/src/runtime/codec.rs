use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use serde::de::DeserializeOwned;
use tokio::sync::oneshot;
use pin_project_lite::pin_project;

use crate::runtime::error::Error;
use crate::runtime::result_source::{LocalResultSource, ResultSource};

pin_project! {
    /// A result source that automatically deserializes JSON responses.
    ///
    /// This wraps a `LocalResultSource<Vec<u8>>` and deserializes the bytes
    /// into the target type `T` when the future completes.
    #[derive(Debug)]
    pub struct JsonResultSource<T> {
        #[pin]
        inner: LocalResultSource<Vec<u8>>,
        _phantom: PhantomData<T>,
    }
}

impl<T> JsonResultSource<T>
where
    T: Send + 'static,
{
    pub(crate) fn from_receiver(rx: oneshot::Receiver<Result<Vec<u8>, Error>>) -> Self {
        Self {
            inner: LocalResultSource::new(rx),
            _phantom: PhantomData,
        }
    }
}

impl<T> Future for JsonResultSource<T>
where
    T: Send + 'static + DeserializeOwned,
{
    type Output = Result<T, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        
        match this.inner.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(bytes)) => {
                match serde_json::from_slice::<T>(&bytes) {
                    Ok(value) => Poll::Ready(Ok(value)),
                    Err(e) => Poll::Ready(Err(Error::Deserialize(e.to_string()))),
                }
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
        }
    }
}

impl<T> ResultSource<T> for JsonResultSource<T>
where
    T: Send + 'static + DeserializeOwned,
{
}

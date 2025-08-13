use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::runtime::{LocalResultSource, TaskHandle, error::Error};

pub type ServiceId = Uuid;
pub type ServiceResult<T> = Result<T, Error>;

/// A trait for defining a long-running, stateful service.
///
/// An implementor of this trait represents an actor in the Actor Model. It owns
/// its own state and processes requests sequentially, guaranteeing that its
/// internal state is accessed safely without data races.
#[async_trait]
pub trait Service: Send {
    /// A method to handle an incoming service call.
    ///
    /// The method takes a mutable reference to `self`, allowing it to modify
    /// the service's internal state. The `payload` is a generic byte buffer,
    /// which should be deserialized by the service implementation.
    ///
    /// # Arguments
    ///
    /// * `method`: The name of the method to be called.
    /// * `payload`: The serialized input data as a byte vector (`Vec<u8>`).
    ///
    /// # Returns
    ///
    /// The serialized output data as a byte vector, or a `ServiceResult` containing an `Error`.
    async fn call(&mut self, method: &str, payload: Vec<u8>) -> ServiceResult<Vec<u8>>;
}

/// A request wrapper that holds the data for a service call and a channel
/// for sending back the response.
///
/// This struct is the "message" in the Actor Model. It is sent from a client
/// (`ServiceAddress`) to the service's runner (`ServiceRunner`) via an `mpsc` channel.
/// The `respond_to` field is a one-shot sender that is created for each call,
/// allowing the runner to send a unique response back to the client.
struct ServiceRequest {
    /// The name of the method to be called on the service.
    method: String,
    /// The serialized input data for the method call.
    payload: Vec<u8>,
    /// The sender half of a one-shot channel to send the service's result back to the caller.
    respond_to: oneshot::Sender<ServiceResult<Vec<u8>>>,
}

/// A sequential runner that owns a service instance and processes requests.
///
/// This struct is the "actor" in the Actor Model. It contains a `Service`
/// and listens for incoming `ServiceRequest`s on its `mpsc` receiver channel.
/// All requests are processed one at a time in the `run` method, which
/// prevents concurrent access to the service's internal state.
pub struct ServiceRunner<S: Service> {
    #[allow(dead_code)] // TODO: remove when we use `id`
    id: ServiceId,
    service: S,
    rx: mpsc::Receiver<ServiceRequest>,
}

impl<S: Service> ServiceRunner<S> {
    /// Spawns a new `ServiceRunner` task and returns a `ServiceAddress` for clients.
    ///
    /// This method creates an `mpsc` channel and a new runner task on the runtime.
    /// The runner takes ownership of the service and the receiver, while the sender
    /// is returned to the caller inside a `ServiceAddress` struct.
    ///
    /// # Arguments
    ///
    /// * `service`: The service instance to be run.
    /// * `capacity`: The capacity of the `mpsc` channel.
    ///
    /// # Returns
    ///
    /// A `ServiceAddress` used to send requests to the running service.
    pub fn spawn(service: S, capacity: usize) -> ServiceAddress
    where
        S: 'static,
    {
        let id = Uuid::new_v4();
        let (tx, rx) = mpsc::channel::<ServiceRequest>(capacity);
        let mut runner = ServiceRunner { id, service, rx };

        tokio::spawn(async move {
            runner.run().await;
        });

        ServiceAddress { id, tx }
    }

    /// The main loop for the service runner.
    ///
    /// This async method continuously awaits a new `ServiceRequest` from the
    /// channel. When a request is received, it calls the service's method,
    /// and sends the result back to the caller.
    async fn run(&mut self) {
        while let Some(req) = self.rx.recv().await {
            let result = self.service.call(&req.method, req.payload).await;
            // Ignore send errors (caller may have dropped the handle)
            let _ = req.respond_to.send(result);
        }
        // When rx is closed, we exit; service is dropped here.
    }
}

/// A public handle for sending requests to a running service.
///
/// This struct acts as a client-side proxy to a `ServiceRunner`. It holds the
/// `mpsc` sender, allowing it to enqueue requests, and can be cloned
/// for use in multiple parts of the application.
#[derive(Clone)]
pub struct ServiceAddress {
    id: ServiceId,
    tx: mpsc::Sender<ServiceRequest>,
}

impl ServiceAddress {
    /// Returns the unique identifier for the service runner.
    pub fn id(&self) -> ServiceId {
        self.id
    }

    /// Enqueues a service call and returns a `TaskHandle` to await the response.
    ///
    /// This method is the primary way to interact with a service. It creates a
    /// new `oneshot` channel for the response and sends a `ServiceRequest`
    /// containing the sender to the service runner.
    ///
    /// # Arguments
    ///
    /// * `method`: The name of the method to call.
    /// * `payload`: The serialized input data.
    ///
    /// # Returns
    ///
    /// A `TaskHandle` that can be awaited to get the `Vec<u8>` response.
    pub fn call(&self, method: &str, payload: Vec<u8>) -> TaskHandle<Vec<u8>> {
        let call_id = Uuid::new_v4();
        let (res_tx, res_rx) = oneshot::channel::<ServiceResult<Vec<u8>>>();

        let req = ServiceRequest {
            method: method.to_string(),
            payload,
            respond_to: res_tx,
        };
        match self.tx.try_send(req) {
            Ok(_) => (),
            Err(err) => {
                let req = err.into_inner();
                let e = if self.tx.is_closed() {
                    Error::ServiceUnavailable
                } else {
                    Error::QueueFull
                };
                let _ = req.respond_to.send(Err(e));
            }
        }

        TaskHandle::new(call_id, LocalResultSource::new(res_rx))
    }
}

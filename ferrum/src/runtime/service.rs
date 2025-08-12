use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::runtime::{LocalResultSource, TaskHandle};

pub type ServiceId = Uuid;
pub type ServiceResult<T> = Result<T, String>;

#[async_trait]
pub trait Service: Send {
    async fn call(&mut self, method: &str, payload: Vec<u8>) -> ServiceResult<Vec<u8>>;
}

// Internal request envelope sent over the mailbox
struct ServiceRequest {
    method: String,
    payload: Vec<u8>,
    respond_to: oneshot::Sender<ServiceResult<Vec<u8>>>,
}

// Sequential runner that owns the service instance
pub struct ServiceRunner<S: Service> {
    id: ServiceId,
    service: S,
    rx: mpsc::Receiver<ServiceRequest>,
}

impl<S: Service> ServiceRunner<S> {
    // Spawn a runner task and return the address for clients to call
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

    async fn run(&mut self) {
        while let Some(req) = self.rx.recv().await {
            let result = self.service.call(&req.method, req.payload).await;
            // Ignore send errors (caller may have dropped the handle)
            let _ = req.respond_to.send(result);
        }
        // When rx is closed, we exit; service is dropped here.
    }
}

// Public, clonable client handle to a running service
#[derive(Clone)]
pub struct ServiceAddress {
    id: ServiceId,
    tx: mpsc::Sender<ServiceRequest>,
}

impl ServiceAddress {
    pub fn id(&self) -> ServiceId {
        self.id
    }

    // Enqueue a call; returns a TaskHandle that resolves to the service's byte response
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
                let _ = req
                    .respond_to
                    .send(Err(format!("Service {id} unavailable", id = self.id)));
            }
        }

        TaskHandle::new(call_id, LocalResultSource::new(res_rx))
    }
}

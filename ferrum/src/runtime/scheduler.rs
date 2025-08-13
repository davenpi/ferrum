use crate::runtime::{Error, handle::TaskHandle, result_source::LocalResultSource, task::Task};
use std::future::Future;
use tokio::sync::oneshot;
use uuid::Uuid;

/// A trait for a task scheduler, responsible for executing tasks on a runtime.
///
/// Implementors of this trait can submit tasks for asynchronous execution.
/// A scheduler must be `Send` and `Sync`, allowing it to be safely shared
/// and used across multiple threads within a runtime.
///
/// # Example
///
/// An example of submitting a task to a scheduler:
///
/// ```ignore
/// struct MyTask;
/// impl Task for MyTask {
///     type Output = ();
///     // ...
/// }
///
/// fn submit_task<S: Scheduler>(scheduler: &S) {
///     let my_task = MyTask;
///     let handle = scheduler.submit(my_task);
///     // The task is now running and can be awaited later
///     // handle.await;
/// }
/// ```
pub trait Scheduler: Send + Sync {
    /// A handle to a submitted task, which can be awaited to get the result.
    ///
    /// This handle must implement `Future` and return a `Result` indicating
    /// either a successful completion with a value of type `T` or an `Error`.
    type Handle<T>: Future<Output = Result<T, Error>> + Send
    where
        T: Send + 'static;

    /// Submits a task to the scheduler for execution.
    ///
    /// The scheduler takes ownership of the `task`. The returned `Handle`
    /// can be used to await the task's completion and retrieve its output.
    ///
    /// # Parameters
    ///
    /// * `task`: The task to be executed. The task must implement `Task`
    ///           and have a `'static` lifetime.
    ///
    /// # Returns
    ///
    /// A `Self::Handle` that can be awaited to get the task's result.
    fn submit<T>(&self, task: T) -> Self::Handle<T::Output>
    where
        T: Task + 'static;
}

// Simple local scheduler
pub struct LocalScheduler {}

impl LocalScheduler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scheduler for LocalScheduler {
    type Handle<T>
        = TaskHandle<T, LocalResultSource<T>>
    where
        T: Send + 'static;

    fn submit<T>(&self, task: T) -> Self::Handle<T::Output>
    where
        T: Task + 'static,
    {
        let task_id = Uuid::new_v4();
        let (sender, receiver) = oneshot::channel::<Result<T::Output, Error>>();

        // Spawn the task execution
        tokio::spawn(async move {
            let result = task.call().await;
            let _ = sender.send(Ok(result));
        });

        TaskHandle::new(task_id, LocalResultSource::new(receiver))
    }
}

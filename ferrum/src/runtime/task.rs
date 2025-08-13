use std::future::Future;
use std::pin::Pin;

/// A trait for defining an asynchronous task that can be executed by the runtime.
///
/// Implementors of this trait represent a single, self-contained unit of work.
/// The `Task` can be sent to and executed on a separate thread, and it owns
/// all of its data or has a `'static` lifetime, ensuring it can live
/// for the duration of the program.
///
/// # Type Parameters
///
/// * `Output`: The type of the value that the task will return upon completion.
///
/// # Example
///
/// ```ignore
/// struct MyTask {
///     data: String,
/// }
///
/// impl Task for MyTask {
///     type Output = String;
///
///     fn call(self) -> Pin<Box<dyn Future<Output = Self::Output> + Send>> {
///         Box::pin(async move {
///             // Asynchronous work here...
///             format!("Processed: {}", self.data)
///         })
///     }
/// }
/// ```
pub trait Task: Send + 'static {
    /// The type of the value that the task will return when it completes.
    type Output: Send + 'static;

    /// Consumes the task and returns a future that, when awaited,
    /// will execute the task's logic and produce an `Output`.
    ///
    /// The returned future is boxed and pinned, allowing it to be
    /// stored on the heap and ensuring its memory location is stable.
    fn call(self) -> Pin<Box<dyn Future<Output = Self::Output> + Send>>;
}

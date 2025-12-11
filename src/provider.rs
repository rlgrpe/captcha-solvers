use crate::errors::RetryableError;
use crate::retry::default_retry_strategy;
use crate::types::TaskId;
use backon::Retryable;
use std::error::Error as StdError;
use std::fmt::{Debug, Display};
use std::future::Future;

#[cfg(feature = "tracing")]
use tracing::debug;

/// Core trait that all captcha solver providers must implement
///
/// This trait uses associated types to allow each provider to define
/// its own task and solution types, enabling type-safe interactions
/// with different captcha solving services.
///
/// # Type Parameters
///
/// - `Task`: The captcha task type specific to this provider (e.g., `CapsolverTask`)
/// - `Solution`: The solution type returned by this provider (e.g., `CapsolverSolution`)
/// - `Error`: The error type for this provider
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::{Provider, TaskId};
///
/// #[derive(Clone)]
/// struct MyProvider { /* ... */ }
///
/// #[derive(Clone)]
/// enum MyTask {
///     Turnstile { url: String, key: String },
///     ReCaptcha { url: String, key: String },
/// }
///
/// struct MySolution {
///     token: String,
/// }
///
/// impl Provider for MyProvider {
///     type Task = MyTask;
///     type Solution = MySolution;
///     type Error = MyError;
///
///     async fn create_task(&self, task: Self::Task) -> Result<TaskId, Self::Error> {
///         // Implementation
///     }
///
///     async fn get_task_result(&self, task_id: &TaskId) -> Result<Option<Self::Solution>, Self::Error> {
///         // Implementation
///     }
/// }
/// ```
pub trait Provider: Send + Sync + Clone {
    /// The captcha task type accepted by this provider
    type Task: Clone + Send + Sync;

    /// The solution type returned by this provider
    type Solution: Send + Sync;

    /// Error type returned by provider operations
    type Error: StdError + RetryableError + Send + Sync + 'static;

    /// Create a new captcha solving task
    ///
    /// # Arguments
    /// * `task` - Task parameters specific to this provider
    ///
    /// # Returns
    /// * `task_id` - Unique identifier for this captcha task
    fn create_task(&self, task: Self::Task) -> impl Future<Output = Result<TaskId, Self::Error>> + Send;

    /// Get the solution for a captcha task if available
    ///
    /// # Arguments
    /// * `task_id` - The task identifier from `create_task`
    ///
    /// # Returns
    /// * `Some(solution)` - Solution if ready
    /// * `None` - Solution not yet ready, caller should poll again
    fn get_task_result(
        &self,
        task_id: &TaskId,
    ) -> impl Future<Output = Result<Option<Self::Solution>, Self::Error>> + Send;
}

/// Wrapper that adds automatic retry logic to any Provider
///
/// This wrapper implements the same `Provider` trait but adds configurable
/// retry behavior based on the error's `is_retryable()` method.
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::{Provider, RetryableProvider};
///
/// let base_provider = CapsolverProvider::new(client);
/// let provider = RetryableProvider::new(base_provider);
///
/// // Now all operations automatically retry on transient errors
/// let task_id = provider.create_task(task).await?;
/// ```
#[derive(Debug, Clone)]
pub struct RetryableProvider<P: Provider> {
    inner: P,
}

impl<P: Provider> RetryableProvider<P> {
    /// Wrap a provider with retry logic
    pub fn new(inner: P) -> Self {
        Self { inner }
    }

    /// Get reference to the inner provider
    pub fn inner(&self) -> &P {
        &self.inner
    }
}

impl<P: Provider> Provider for RetryableProvider<P>
where
    P::Task: Display,
    P::Error: Debug + Display,
{
    type Task = P::Task;
    type Solution = P::Solution;
    type Error = P::Error;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(name = "RetryableProvider::create_task", skip_all, fields(task_type))
    )]
    async fn create_task(&self, task: Self::Task) -> Result<TaskId, Self::Error> {
        #[cfg(feature = "tracing")]
        tracing::Span::current().record("task_type", task.to_string());

        let inner = self.inner.clone();
        let task_clone = task.clone();
        (|| async { inner.create_task(task_clone.clone()).await })
            .retry(default_retry_strategy())
            .when(|err: &Self::Error| err.is_retryable())
            .notify(|err, duration| {
                let _ = (err, duration);
                #[cfg(feature = "tracing")]
                debug!(
                    error = %err,
                    task_type = %task,
                    retry_after_secs = %duration.as_secs_f64(),
                    "Retrying create_task after transient error"
                );
            })
            .await
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "RetryableProvider::get_task_result",
            skip_all,
            fields(task_id = %task_id)
        )
    )]
    async fn get_task_result(
        &self,
        task_id: &TaskId,
    ) -> Result<Option<Self::Solution>, Self::Error> {
        let inner = self.inner.clone();
        let task_id = task_id.clone();
        (|| async { inner.get_task_result(&task_id).await })
            .retry(default_retry_strategy())
            .when(|err: &Self::Error| err.is_retryable())
            .notify(|err, duration| {
                let _ = (err, duration);
                #[cfg(feature = "tracing")]
                debug!(
                    error = %err,
                    task_id = %task_id,
                    retry_after_secs = %duration.as_secs_f64(),
                    "Retrying get_task_result after transient error"
                );
            })
            .await
    }
}
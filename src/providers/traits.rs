//! Core provider trait definition.

#![allow(async_fn_in_trait)]

use crate::errors::RetryableError;
use crate::solutions::ProviderSolution;
use crate::tasks::CaptchaTask;
use crate::utils::types::TaskId;
use std::error::Error as StdError;

/// Core trait that all captcha solver providers must implement.
///
/// This trait uses [`CaptchaTask`] as a unified input type for all captcha tasks,
/// allowing providers to internally convert to their specific format.
///
/// # Type Parameters
///
/// - `Solution`: The solution type returned by this provider (e.g., `CapsolverSolution`)
/// - `Error`: The error type for this provider
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::{Provider, CaptchaTask, TaskId, ProviderSolution};
///
/// #[derive(Clone)]
/// struct MyProvider { /* ... */ }
///
/// #[derive(Debug, Clone)]
/// struct MySolution { token: String }
///
/// impl ProviderSolution for MySolution {}
///
/// impl Provider for MyProvider {
///     type Solution = MySolution;
///     type Error = MyError;
///
///     async fn create_task(&self, task: CaptchaTask) -> Result<TaskId, Self::Error> {
///         // Convert CaptchaTask to internal format and submit
///     }
///
///     async fn get_task_result(&self, task_id: &TaskId) -> Result<Option<Self::Solution>, Self::Error> {
///         // Poll for solution
///     }
/// }
/// ```
pub trait Provider: Send + Sync + Clone {
    /// The solution type returned by this provider.
    ///
    /// Must implement [`ProviderSolution`] for compatibility with [`CaptchaSolverService`].
    type Solution: ProviderSolution;

    /// Error type returned by provider operations.
    ///
    /// Must implement [`RetryableError`] to classify errors as transient or permanent.
    type Error: StdError + RetryableError + Send + Sync + 'static;

    /// Create a new captcha solving task.
    ///
    /// # Arguments
    ///
    /// * `task` - Unified captcha task (will be converted internally to provider-specific format)
    ///
    /// # Returns
    ///
    /// * `Ok(task_id)` - Unique identifier for the created task
    /// * `Err(error)` - If task creation failed
    async fn create_task(&self, task: CaptchaTask) -> Result<TaskId, Self::Error>;

    /// Get the solution for a captcha task if available.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task identifier from `create_task`
    ///
    /// # Returns
    ///
    /// * `Ok(Some(solution))` - Solution is ready
    /// * `Ok(None)` - Solution not yet ready, caller should poll again
    /// * `Err(error)` - If polling failed
    async fn get_task_result(
        &self,
        task_id: &TaskId,
    ) -> Result<Option<Self::Solution>, Self::Error>;
}

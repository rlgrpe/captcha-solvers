//! Core provider trait definition.

#![allow(async_fn_in_trait)]

use crate::errors::RetryableError;
use crate::solutions::ProviderSolution;
use crate::tasks::CaptchaTask;
use crate::utils::types::TaskId;
use std::error::Error as StdError;

/// Result of creating a captcha task.
///
/// Some task types (e.g., `ImageToText` on Capsolver) return the solution
/// immediately without requiring polling. This enum allows providers to
/// express both patterns cleanly.
#[derive(Debug, Clone)]
pub enum TaskCreationOutcome<S> {
    /// Task was created and requires polling via `get_task_result`.
    Pending(TaskId),

    /// Task completed immediately with solution (no polling needed).
    ///
    /// This is used for synchronous task types like `ImageToText` on Capsolver,
    /// where the API returns the solution directly in the `createTask` response.
    Ready {
        /// The task identifier (may still be useful for logging/debugging).
        task_id: TaskId,
        /// The solution returned immediately.
        solution: S,
    },
}

impl<S> TaskCreationOutcome<S> {
    /// Returns the task ID regardless of outcome type.
    pub fn task_id(&self) -> &TaskId {
        match self {
            Self::Pending(id) => id,
            Self::Ready { task_id, .. } => task_id,
        }
    }

    /// Returns `true` if the task is pending (requires polling).
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending(_))
    }

    /// Returns `true` if the task completed immediately with a solution.
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    /// Extract the solution if immediately ready.
    pub fn into_solution(self) -> Option<S> {
        match self {
            Self::Ready { solution, .. } => Some(solution),
            Self::Pending(_) => None,
        }
    }
}

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
/// use captcha_solvers::{Provider, CaptchaTask, TaskCreationOutcome, ProviderSolution};
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
///     async fn create_task(&self, task: CaptchaTask) -> Result<TaskCreationOutcome<Self::Solution>, Self::Error> {
///         // Convert CaptchaTask to internal format and submit
///         // Return Pending(task_id) or Ready { task_id, solution }
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
    /// Some task types (like `ImageToText` on Capsolver) return the solution
    /// immediately without requiring polling. Check the returned
    /// [`TaskCreationOutcome`] to determine whether polling is needed.
    ///
    /// # Arguments
    ///
    /// * `task` - Unified captcha task (will be converted internally to provider-specific format)
    ///
    /// # Returns
    ///
    /// * `Ok(Pending(task_id))` - Task created, poll with `get_task_result`
    /// * `Ok(Ready { task_id, solution })` - Task completed immediately
    /// * `Err(error)` - If task creation failed
    async fn create_task(
        &self,
        task: CaptchaTask,
    ) -> Result<TaskCreationOutcome<Self::Solution>, Self::Error>;

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

//! Service-level error types.

use crate::errors::RetryableError;
use crate::utils::types::TaskId;
use std::error::Error as StdError;
use std::time::Duration;
use thiserror::Error;

/// Service-level errors that wrap provider errors.
///
/// This error type uses a boxed error for the provider source, allowing it to
/// work with any provider implementation without generic type parameters.
///
/// # Retryability
///
/// The error implements [`RetryableError`] with two levels:
///
/// - `is_retryable()`: Whether the same task can be retried (e.g., network timeout)
/// - `should_retry_operation()`: Whether a fresh solve attempt might work (e.g., captcha unsolvable)
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::{ServiceError, RetryableError};
///
/// match result {
///     Err(e) if e.is_retryable() => { /* retry same task */ }
///     Err(e) if e.should_retry_operation() => { /* try fresh task */ }
///     Err(e) => { /* permanent failure */ }
///     Ok(solution) => { /* success */ }
/// }
/// ```
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Error from the underlying captcha provider.
    #[error("Captcha provider error: {source}")]
    Provider {
        /// The original provider error.
        #[source]
        source: Box<dyn StdError + Send + Sync>,
        /// Whether the same task can be retried.
        is_retryable: bool,
        /// Whether a fresh solve operation might succeed.
        should_retry_operation: bool,
    },

    /// Timeout waiting for captcha solution.
    #[error(
        "Timeout waiting for captcha solution after {:.1}s; Task id: {task_id}",
        timeout.as_secs_f64()
    )]
    SolutionTimeout {
        /// The timeout duration that was exceeded.
        timeout: Duration,
        /// The task ID that timed out.
        task_id: TaskId,
    },
}

impl ServiceError {
    /// Create a new provider error from any error that implements the standard error trait.
    ///
    /// This method captures the retryability flags from the error if it implements
    /// [`RetryableError`], otherwise defaults to non-retryable.
    pub fn from_provider<E>(error: E) -> Self
    where
        E: StdError + RetryableError + Send + Sync + 'static,
    {
        let is_retryable = error.is_retryable();
        let should_retry_operation = error.should_retry_operation();
        Self::Provider {
            source: Box::new(error),
            is_retryable,
            should_retry_operation,
        }
    }

    /// Create a solution timeout error.
    pub fn timeout(timeout: Duration, task_id: TaskId) -> Self {
        Self::SolutionTimeout { timeout, task_id }
    }
}

impl RetryableError for ServiceError {
    fn is_retryable(&self) -> bool {
        match self {
            ServiceError::Provider { is_retryable, .. } => *is_retryable,
            // Can't retry the same task after timeout - it's expired
            ServiceError::SolutionTimeout { .. } => false,
        }
    }

    fn should_retry_operation(&self) -> bool {
        match self {
            ServiceError::Provider {
                should_retry_operation,
                ..
            } => *should_retry_operation,
            // A fresh task attempt might succeed after timeout
            ServiceError::SolutionTimeout { .. } => true,
        }
    }
}

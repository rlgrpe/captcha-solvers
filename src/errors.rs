use std::fmt;
use thiserror::Error;

/// Error for task types not supported by a provider.
///
/// This error is returned when attempting to convert a shared task type
/// to a provider-specific format that doesn't support that task type.
///
/// # Example
///
/// ```rust
/// use captcha_solvers::UnsupportedTaskError;
///
/// let error = UnsupportedTaskError::new("CloudflareChallenge", "RuCaptcha");
/// assert_eq!(
///     error.to_string(),
///     "Task type 'CloudflareChallenge' is not supported by RuCaptcha. \
///      This task type is only available with other providers."
/// );
/// ```
#[derive(Debug, Clone, Error)]
pub struct UnsupportedTaskError {
    /// The task type that is not supported
    pub task_type: &'static str,
    /// The provider name that doesn't support this task
    pub provider: &'static str,
}

impl UnsupportedTaskError {
    /// Create a new unsupported task error.
    pub fn new(task_type: &'static str, provider: &'static str) -> Self {
        Self {
            task_type,
            provider,
        }
    }
}

impl fmt::Display for UnsupportedTaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Task type '{}' is not supported by {}. \
             This task type is only available with other providers.",
            self.task_type, self.provider
        )
    }
}

/// Trait for errors that can be classified as retryable or permanent
///
/// This trait provides two levels of retryability classification:
///
/// 1. **Task-level** (`is_retryable`): Whether the same task_id operation should be retried.
///    Use this for transient errors like network timeouts or rate limits.
///
/// 2. **Operation-level** (`should_retry_operation`): Whether a fresh solve attempt
///    (creating a new task) might succeed. Use this when a specific captcha task
///    failed but trying again with a new task could work.
///
/// # Examples
///
/// ```rust
/// use captcha_solvers::RetryableError;
///
/// enum MyError {
///     NetworkTimeout,      // Retry same task
///     CaptchaUnsolvable,   // Don't retry task, but try fresh operation
///     InvalidApiKey,       // Don't retry at all
///     ZeroBalance,         // Don't retry until account is funded
/// }
///
/// impl RetryableError for MyError {
///     fn is_retryable(&self) -> bool {
///         match self {
///             MyError::NetworkTimeout => true,
///             _ => false,
///         }
///     }
///
///     fn should_retry_operation(&self) -> bool {
///         match self {
///             MyError::NetworkTimeout => true,
///             MyError::CaptchaUnsolvable => true,  // Fresh attempt might work
///             MyError::InvalidApiKey => false,     // Won't ever work
///             MyError::ZeroBalance => false,       // Need to fund account first
///         }
///     }
/// }
/// ```
pub trait RetryableError {
    /// Returns true if this error represents a transient failure
    /// that might succeed on retry with the same task_id.
    ///
    /// Examples: network timeouts, rate limits, temporary service unavailability.
    fn is_retryable(&self) -> bool;

    /// Returns true if a fresh solve operation (creating a new task) might succeed.
    ///
    /// This is useful when a specific captcha task failed (e.g., captcha was unsolvable)
    /// but trying again with a completely new task might work.
    ///
    /// Default implementation returns the same as `is_retryable()`.
    ///
    /// Examples where this differs from `is_retryable()`:
    /// - `CaptchaUnsolvable`: is_retryable=false, should_retry_operation=true
    /// - `TaskTimeout`: is_retryable=false, should_retry_operation=true
    /// - `ZeroBalance`: is_retryable=false, should_retry_operation=false
    fn should_retry_operation(&self) -> bool {
        self.is_retryable()
    }
}

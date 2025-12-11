/// Trait for errors that can be classified as retryable or permanent
///
/// This trait allows error types to specify whether a failed operation
/// should be retried or if it's a permanent failure that won't succeed
/// even with retries.
///
/// # Examples
///
/// ```rust
/// use captcha_solvers::errors::RetryableError;
///
/// enum MyError {
///     NetworkTimeout,
///     InvalidApiKey,
///     ServiceUnavailable,
/// }
///
/// impl RetryableError for MyError {
///     fn is_retryable(&self) -> bool {
///         match self {
///             // Transient errors - worth retrying
///             MyError::NetworkTimeout => true,
///             MyError::ServiceUnavailable => true,
///             // Permanent errors - no point retrying
///             MyError::InvalidApiKey => false,
///         }
///     }
/// }
/// ```
pub trait RetryableError {
    /// Returns true if this error represents a transient failure
    /// that might succeed on retry, false for permanent failures.
    fn is_retryable(&self) -> bool;
}

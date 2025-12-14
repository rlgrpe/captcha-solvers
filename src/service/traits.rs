//! Captcha solver service trait definition.

#![allow(async_fn_in_trait)]

use super::errors::ServiceError;
use crate::solutions::ProviderSolution;
use crate::tasks::CaptchaTask;
use tokio_util::sync::CancellationToken;

/// Trait for captcha solver service implementations.
///
/// This trait abstracts the service interface, allowing different
/// service implementations to be used interchangeably.
///
/// # Usage Patterns
///
/// The service provides a unified interface for solving captchas:
///
/// ```rust,ignore
/// use captcha_solvers::{
///     CaptchaSolverService, CaptchaSolverServiceTrait,
///     ReCaptchaV2, capsolver::CapsolverProvider,
/// };
/// use std::time::Duration;
///
/// let provider = CapsolverProvider::new("api_key")?;
/// let service = CaptchaSolverService::new(provider);
///
/// // Use the trait method
/// let solution = service
///     .solve_captcha(ReCaptchaV2::new("https://example.com", "site_key"))
///     .await?;
/// ```
pub trait CaptchaSolverServiceTrait: Send + Sync {
    /// The solution type returned by this service.
    ///
    /// This is typically a provider-specific solution type that can be
    /// converted to the appropriate captcha solution (ReCaptcha, Turnstile, etc.).
    type Solution: ProviderSolution;

    /// Solve a captcha task using the service's default timeout.
    ///
    /// This method handles the complete captcha solving lifecycle:
    /// 1. Creates a task with the provider
    /// 2. Polls for the solution until ready or timeout
    /// 3. Returns the solution or an error
    ///
    /// # Arguments
    ///
    /// * `task` - Any type that can be converted to [`CaptchaTask`]
    ///
    /// # Returns
    ///
    /// * `Ok(Solution)` - The captcha solution
    /// * `Err(ServiceError)` - If the task failed or timed out
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use captcha_solvers::{ReCaptchaV2, Turnstile};
    ///
    /// // ReCaptcha V2
    /// let task = ReCaptchaV2::new("https://example.com", "site_key")
    ///     .invisible()
    ///     .enterprise();
    /// let solution = service.solve_captcha(task).await?;
    /// let token = solution.into_recaptcha().token();
    ///
    /// // Turnstile
    /// let task = Turnstile::new("https://example.com", "site_key");
    /// let solution = service.solve_captcha(task).await?;
    /// let token = solution.into_turnstile().token();
    /// ```
    async fn solve_captcha<T: Into<CaptchaTask> + Send>(
        &self,
        task: T,
    ) -> Result<Self::Solution, ServiceError>;

    /// Solve a captcha task with cancellation support.
    ///
    /// This method is similar to [`solve_captcha`](Self::solve_captcha) but accepts
    /// a [`CancellationToken`] that can be used to cancel the operation.
    ///
    /// # Arguments
    ///
    /// * `task` - Any type that can be converted to [`CaptchaTask`]
    /// * `cancel_token` - Token to cancel the operation
    ///
    /// # Returns
    ///
    /// * `Ok(Solution)` - The captcha solution
    /// * `Err(ServiceError::Cancelled)` - If the operation was cancelled
    /// * `Err(ServiceError)` - If the task failed or timed out
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use captcha_solvers::ReCaptchaV2;
    /// use tokio_util::sync::CancellationToken;
    ///
    /// let cancel_token = CancellationToken::new();
    /// let token_clone = cancel_token.clone();
    ///
    /// // Spawn a task that will cancel after 30 seconds
    /// tokio::spawn(async move {
    ///     tokio::time::sleep(Duration::from_secs(30)).await;
    ///     token_clone.cancel();
    /// });
    ///
    /// let task = ReCaptchaV2::new("https://example.com", "site_key");
    /// match service.solve_captcha_cancellable(task, cancel_token).await {
    ///     Ok(solution) => println!("Got solution: {}", solution.into_recaptcha().token()),
    ///     Err(e) if e.is_cancelled() => println!("Operation was cancelled"),
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// ```
    async fn solve_captcha_cancellable<T: Into<CaptchaTask> + Send>(
        &self,
        task: T,
        cancel_token: CancellationToken,
    ) -> Result<Self::Solution, ServiceError>;
}

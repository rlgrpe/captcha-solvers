//! Captcha solver service trait definition.

use super::errors::ServiceError;
use crate::solutions::ProviderSolution;
use crate::tasks::CaptchaTask;
use std::time::Duration;

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
///     .solve_captcha(ReCaptchaV2::new("https://example.com", "site_key"), Duration::from_secs(120))
///     .await?;
/// ```
pub trait CaptchaSolverServiceTrait: Send + Sync {
    /// The solution type returned by this service.
    ///
    /// This is typically a provider-specific solution type that can be
    /// converted to the appropriate captcha solution (ReCaptcha, Turnstile, etc.).
    type Solution: ProviderSolution;

    /// Solve a captcha task.
    ///
    /// This method handles the complete captcha solving lifecycle:
    /// 1. Creates a task with the provider
    /// 2. Polls for the solution until ready or timeout
    /// 3. Returns the solution or an error
    ///
    /// # Arguments
    ///
    /// * `task` - Any type that can be converted to [`CaptchaTask`]
    /// * `timeout` - Maximum time to wait for the solution
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
    /// use std::time::Duration;
    ///
    /// // ReCaptcha V2
    /// let task = ReCaptchaV2::new("https://example.com", "site_key")
    ///     .invisible()
    ///     .enterprise();
    /// let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;
    /// let token = solution.into_recaptcha().token();
    ///
    /// // Turnstile
    /// let task = Turnstile::new("https://example.com", "site_key");
    /// let solution = service.solve_captcha(task, Duration::from_secs(60)).await?;
    /// let token = solution.into_turnstile().token();
    /// ```
    async fn solve_captcha<T: Into<CaptchaTask> + Send>(
        &self,
        task: T,
        timeout: Duration,
    ) -> Result<Self::Solution, ServiceError>;
}

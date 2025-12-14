//! Core captcha solver service implementation.

use super::config::CaptchaSolverServiceConfig;
use super::errors::ServiceError;
use super::traits::CaptchaSolverServiceTrait;
use crate::errors::RetryableError;
use crate::providers::traits::Provider;
use crate::tasks::CaptchaTask;
use std::fmt::{Debug, Display};
use std::time::Instant;

#[cfg(feature = "tracing")]
use tracing::{Span, error, info, warn};

/// Generic captcha solver service that works with any Provider implementation.
///
/// This service handles high-level captcha solving operations:
/// - Creating captcha tasks
/// - Polling for solutions with timeout
/// - Managing task lifecycle
/// - Error handling and classification
///
/// The actual captcha provider logic is abstracted behind the [`Provider`] trait.
///
/// # Type Parameters
///
/// - `P`: The provider implementation (e.g., `CapsolverProvider`, `RucaptchaProvider`)
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::{
///     CaptchaSolverService, CaptchaSolverServiceTrait,
///     ReCaptchaV2, Turnstile,
///     capsolver::CapsolverProvider,
/// };
/// use std::time::Duration;
///
/// // Create provider and service
/// let provider = CapsolverProvider::new("api_key")?;
/// let service = CaptchaSolverService::new(provider);
///
/// // Solve using shared task types
/// let task = ReCaptchaV2::new("https://example.com", "site_key")
///     .invisible()
///     .enterprise();
///
/// let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;
/// println!("Token: {}", solution.into_recaptcha().token());
/// ```
///
/// # With Retry Support
///
/// Wrap the provider with [`RetryableProvider`](crate::RetryableProvider) for automatic retries:
///
/// ```rust,ignore
/// use captcha_solvers::{RetryableProvider, RetryConfig};
///
/// let provider = CapsolverProvider::new("api_key")?;
/// let retryable = RetryableProvider::with_config(provider, RetryConfig::default());
/// let service = CaptchaSolverService::new(retryable);
/// ```
#[derive(Debug, Clone)]
pub struct CaptchaSolverService<P: Provider> {
    provider: P,
    config: CaptchaSolverServiceConfig,
}

impl<P: Provider> CaptchaSolverService<P>
where
    P::Error: Debug + Display + RetryableError,
{
    /// Create a new captcha solver service with the given provider.
    ///
    /// Uses the default configuration with a 3 second poll interval.
    ///
    /// # Arguments
    ///
    /// * `provider` - The captcha provider implementation
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use captcha_solvers::{CaptchaSolverService, capsolver::CapsolverProvider};
    ///
    /// let provider = CapsolverProvider::new("api_key")?;
    /// let service = CaptchaSolverService::new(provider);
    /// ```
    pub fn new(provider: P) -> Self {
        Self::with_config(provider, CaptchaSolverServiceConfig::default())
    }

    /// Create a new captcha solver service with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `provider` - The captcha provider implementation
    /// * `config` - Service configuration
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceConfig};
    /// use std::time::Duration;
    ///
    /// let config = CaptchaSolverServiceConfig {
    ///     poll_interval: Duration::from_secs(5),
    /// };
    /// let service = CaptchaSolverService::with_config(provider, config);
    /// ```
    pub fn with_config(provider: P, config: CaptchaSolverServiceConfig) -> Self {
        Self { provider, config }
    }

    /// Get a reference to the underlying provider.
    ///
    /// Useful for accessing provider-specific methods or configuration.
    pub fn provider(&self) -> &P {
        &self.provider
    }

    /// Get a reference to the service configuration.
    pub fn config(&self) -> &CaptchaSolverServiceConfig {
        &self.config
    }
}

impl<P: Provider> CaptchaSolverServiceTrait for CaptchaSolverService<P>
where
    P::Error: Debug + Display + RetryableError + 'static,
{
    type Solution = P::Solution;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "captcha.solve",
            skip_all,
            fields(
                captcha.task_type,
                captcha.task_id,
                captcha.provider = std::any::type_name::<P>()
            )
        )
    )]
    async fn solve_captcha<T: Into<CaptchaTask> + Send>(
        &self,
        task: T,
        timeout: std::time::Duration,
    ) -> Result<Self::Solution, ServiceError> {
        let task = task.into();

        #[cfg(feature = "tracing")]
        Span::current().record("captcha.task_type", task.to_string());

        // Create the task
        let task_id = self
            .provider
            .create_task(task)
            .await
            .map_err(ServiceError::from_provider)?;

        #[cfg(feature = "tracing")]
        {
            Span::current().record("captcha.task_id", task_id.as_ref());
            info!("Captcha task created, polling for solution");
        }

        // Poll for solution with timeout
        let poll_interval = self.config.poll_interval;
        let start = Instant::now();

        loop {
            if start.elapsed() >= timeout {
                #[cfg(feature = "tracing")]
                warn!(
                    elapsed_secs = %start.elapsed().as_secs_f64(),
                    timeout_secs = %timeout.as_secs_f64(),
                    "Captcha solution timeout"
                );
                return Err(ServiceError::timeout(timeout, task_id));
            }

            match self.provider.get_task_result(&task_id).await {
                Ok(Some(solution)) => {
                    #[cfg(feature = "tracing")]
                    info!(
                        elapsed_secs = %start.elapsed().as_secs_f64(),
                        "Captcha solved successfully"
                    );
                    return Ok(solution);
                }
                Ok(None) => {
                    // Solution not yet ready, continue polling
                }
                Err(e) if !e.is_retryable() => {
                    // Permanent error - return immediately
                    #[cfg(feature = "tracing")]
                    error!(
                        error = %e,
                        "Permanent error while polling for solution"
                    );
                    return Err(ServiceError::from_provider(e));
                }
                Err(_e) => {
                    // Transient error - log and continue polling
                    #[cfg(feature = "tracing")]
                    warn!(
                        error = %_e,
                        "Transient error while polling, will retry"
                    );
                }
            }

            tokio::time::sleep(poll_interval).await;
        }
    }
}

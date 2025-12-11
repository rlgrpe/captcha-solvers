use crate::errors::RetryableError;
use crate::provider::Provider;
use crate::types::TaskId;
use snafu::Snafu;
use std::error::Error as StdError;
use std::fmt::{Debug, Display};
use std::future::Future;
use std::time::{Duration, Instant};

#[cfg(feature = "tracing")]
use snafu::Report;
#[cfg(feature = "tracing")]
use tracing::{error, info, warn, Span};

/// Service-level errors that wrap provider errors
#[derive(Debug, Snafu)]
pub enum ServiceError<E: StdError + 'static> {
    #[snafu(display("Captcha provider error"))]
    Provider {
        source: E,
        is_retryable: bool,
    },

    #[snafu(display(
        "Timeout waiting for captcha solution after {:.1}s; Task id: {}",
        timeout.as_secs_f64(),
        task_id
    ))]
    SolutionTimeout {
        timeout: Duration,
        task_id: TaskId,
    },
}

impl<E: StdError + 'static> RetryableError for ServiceError<E> {
    fn is_retryable(&self) -> bool {
        match self {
            ServiceError::Provider { is_retryable, .. } => *is_retryable,
            ServiceError::SolutionTimeout { .. } => true,
        }
    }
}

/// Trait for captcha solver service implementations
///
/// This trait abstracts the service interface, allowing different
/// service implementations to be used interchangeably.
pub trait CaptchaSolverServiceTrait: Send + Sync {
    /// The task type accepted by this service
    type Task;
    /// The solution type returned by this service
    type Solution;
    /// The error type for this service
    type Error: StdError + RetryableError;

    /// Solve a captcha task
    fn solve_captcha(
        &self,
        task: Self::Task,
        timeout: Duration,
    ) -> impl Future<Output = Result<Self::Solution, Self::Error>> + Send;
}

/// Configuration for Captcha Solver Service
#[derive(Debug, Clone)]
pub struct CaptchaSolverServiceConfig {
    /// Interval between polling attempts when waiting for solution
    pub poll_interval: Duration,
}

impl Default for CaptchaSolverServiceConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(3),
        }
    }
}

/// Generic captcha solver service that works with any Provider implementation
///
/// This service handles high-level captcha solving operations like:
/// - Creating captcha tasks
/// - Polling for solutions with timeout
/// - Managing task lifecycle
///
/// The actual captcha provider logic is abstracted behind the `Provider` trait.
///
/// # Type Parameters
///
/// - `P`: The provider implementation (e.g., `CapsolverProvider`, `TwoCaptchaProvider`)
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceConfig, Provider};
/// use std::time::Duration;
///
/// let provider = MyProvider::new(api_key);
/// let config = CaptchaSolverServiceConfig {
///     poll_interval: Duration::from_secs(3),
/// };
///
/// let service = CaptchaSolverService::new(provider, config);
///
/// let task = MyTask::Turnstile {
///     website_url: "https://example.com".to_string(),
///     website_key: "site_key".to_string(),
/// };
///
/// let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;
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
    /// Create a new captcha solver service with a custom provider and configuration
    pub fn new(provider: P, config: CaptchaSolverServiceConfig) -> Self {
        Self { provider, config }
    }

    /// Get reference to the underlying provider
    pub fn provider(&self) -> &P {
        &self.provider
    }

    /// Get reference to the service configuration
    pub fn config(&self) -> &CaptchaSolverServiceConfig {
        &self.config
    }
}

impl<P: Provider> CaptchaSolverServiceTrait for CaptchaSolverService<P>
where
    P::Task: Display,
    P::Error: Debug + Display + RetryableError + 'static,
{
    type Task = P::Task;
    type Solution = P::Solution;
    type Error = ServiceError<P::Error>;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "CaptchaSolverService::solve_captcha",
            skip_all,
            fields(task_type)
        )
    )]
    async fn solve_captcha(
        &self,
        task: Self::Task,
        timeout: Duration,
    ) -> Result<Self::Solution, Self::Error> {
        #[cfg(feature = "tracing")]
        Span::current().record("task_type", task.to_string());

        // Create the task
        let task_id = self.provider.create_task(task).await.map_err(|e| {
            let is_retryable = e.is_retryable();
            ServiceError::Provider {
                source: e,
                is_retryable,
            }
        })?;

        #[cfg(feature = "tracing")]
        info!(
            task_id = %task_id,
            "Captcha task created, starting to poll for solution"
        );

        // Poll for solution with timeout
        let poll_interval = self.config.poll_interval;
        let start = Instant::now();

        loop {
            if start.elapsed() >= timeout {
                #[cfg(feature = "tracing")]
                warn!(
                    timeout_secs = %timeout.as_secs_f64(),
                    "Captcha solution timeout"
                );
                return Err(ServiceError::SolutionTimeout {
                    timeout,
                    task_id: task_id.clone(),
                });
            }

            match self.provider.get_task_result(&task_id).await {
                Ok(Some(solution)) => {
                    #[cfg(feature = "tracing")]
                    info!(
                        elapsed_secs = %start.elapsed().as_secs_f64(),
                        "Captcha solution received successfully"
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
                        error = %Report::from_error(&e),
                        "Permanent error while polling for captcha solution"
                    );
                    return Err(ServiceError::Provider {
                        source: e,
                        is_retryable: false,
                    });
                }
                Err(_e) => {
                    // Transient error - log and continue
                    #[cfg(feature = "tracing")]
                    warn!(
                        error = %Report::from_error(&_e),
                        "Transient error while polling for captcha solution"
                    );
                }
            }

            tokio::time::sleep(poll_interval).await;
        }
    }
}
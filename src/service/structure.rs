//! Core captcha solver service implementation.

use super::config::{CaptchaSolverServiceConfig, CaptchaSolverServiceConfigBuilder};
use super::errors::ServiceError;
use super::traits::CaptchaSolverServiceTrait;
use crate::errors::RetryableError;
use crate::providers::traits::{Provider, TaskCreationOutcome};
use crate::tasks::CaptchaTask;
use std::fmt::{Debug, Display};
use std::time::Instant;
use tokio_util::sync::CancellationToken;

#[cfg(feature = "tracing")]
use tracing::{Span, debug, error, info, warn};

#[cfg(feature = "metrics")]
use opentelemetry::{
    KeyValue, global,
    metrics::{Counter, Histogram},
};

#[cfg(feature = "metrics")]
use std::sync::OnceLock;

/// Metrics for the captcha solver service.
#[cfg(feature = "metrics")]
struct ServiceMetrics {
    /// Counter for task creation requests.
    tasks_created: Counter<u64>,
    /// Counter for successful solutions.
    solutions_received: Counter<u64>,
    /// Counter for timeouts.
    timeouts: Counter<u64>,
    /// Counter for cancellations.
    cancellations: Counter<u64>,
    /// Counter for errors.
    errors: Counter<u64>,
    /// Histogram for solve times in seconds.
    solve_time: Histogram<f64>,
    /// Histogram for poll counts.
    poll_counts: Histogram<u64>,
}

#[cfg(feature = "metrics")]
impl ServiceMetrics {
    fn global() -> &'static Self {
        static METRICS: OnceLock<ServiceMetrics> = OnceLock::new();
        METRICS.get_or_init(|| {
            let meter = global::meter("captcha_solvers");
            Self {
                tasks_created: meter
                    .u64_counter("captcha_solvers.tasks_created")
                    .with_description("Number of captcha tasks created")
                    .build(),
                solutions_received: meter
                    .u64_counter("captcha_solvers.solutions_received")
                    .with_description("Number of captcha solutions successfully received")
                    .build(),
                timeouts: meter
                    .u64_counter("captcha_solvers.timeouts")
                    .with_description("Number of captcha solve timeouts")
                    .build(),
                cancellations: meter
                    .u64_counter("captcha_solvers.cancellations")
                    .with_description("Number of cancelled captcha operations")
                    .build(),
                errors: meter
                    .u64_counter("captcha_solvers.errors")
                    .with_description("Number of captcha solve errors")
                    .build(),
                solve_time: meter
                    .f64_histogram("captcha_solvers.solve_time_seconds")
                    .with_description("Time spent solving captchas")
                    .build(),
                poll_counts: meter
                    .u64_histogram("captcha_solvers.poll_counts")
                    .with_description("Number of polls before receiving solution")
                    .build(),
            }
        })
    }
}

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
/// let solution = service.solve_captcha(task).await?;
/// println!("Token: {}", solution.into_recaptcha().token());
/// ```
///
/// # With Builder Pattern
///
/// ```rust,ignore
/// use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceConfig};
/// use std::time::Duration;
///
/// let service = CaptchaSolverService::builder(provider)
///     .timeout(Duration::from_secs(180))
///     .poll_interval(Duration::from_secs(5))
///     .build();
/// ```
///
/// # With Retry Support
///
/// Wrap the provider with [`CaptchaRetryableProvider`](crate::CaptchaRetryableProvider) for automatic retries:
///
/// ```rust,ignore
/// use captcha_solvers::{CaptchaRetryableProvider, RetryConfig};
///
/// let provider = CapsolverProvider::new("api_key")?;
/// let retryable = CaptchaRetryableProvider::with_config(provider, RetryConfig::default());
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
    /// Uses the default configuration (balanced preset):
    /// - Timeout: 120 seconds
    /// - Poll interval: 3 seconds
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
    ///
    /// let config = CaptchaSolverServiceConfig::fast();
    /// let service = CaptchaSolverService::with_config(provider, config);
    /// ```
    pub fn with_config(provider: P, config: CaptchaSolverServiceConfig) -> Self {
        Self { provider, config }
    }

    /// Create a new builder for CaptchaSolverService.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use captcha_solvers::CaptchaSolverService;
    /// use std::time::Duration;
    ///
    /// let service = CaptchaSolverService::builder(provider)
    ///     .timeout(Duration::from_secs(180))
    ///     .poll_interval(Duration::from_secs(5))
    ///     .build();
    /// ```
    pub fn builder(provider: P) -> CaptchaSolverServiceBuilder<P> {
        CaptchaSolverServiceBuilder::new(provider)
    }

    /// Get a reference to the underlying provider.
    ///
    /// Useful for accessing provider-specific methods or configuration.
    pub fn provider(&self) -> &P {
        &self.provider
    }

    /// Get a mutable reference to the underlying provider.
    pub fn provider_mut(&mut self) -> &mut P {
        &mut self.provider
    }

    /// Get a reference to the service configuration.
    pub fn config(&self) -> &CaptchaSolverServiceConfig {
        &self.config
    }

    /// Get a mutable reference to the service configuration.
    pub fn config_mut(&mut self) -> &mut CaptchaSolverServiceConfig {
        &mut self.config
    }

    /// Update the service configuration.
    pub fn set_config(&mut self, config: CaptchaSolverServiceConfig) {
        self.config = config;
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
    ) -> Result<Self::Solution, ServiceError> {
        self.solve_captcha_cancellable(task, CancellationToken::new())
            .await
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "captcha.solve_cancellable",
            skip_all,
            fields(
                captcha.task_type,
                captcha.task_id,
                captcha.provider = std::any::type_name::<P>()
            )
        )
    )]
    async fn solve_captcha_cancellable<T: Into<CaptchaTask> + Send>(
        &self,
        task: T,
        cancel_token: CancellationToken,
    ) -> Result<Self::Solution, ServiceError> {
        let task = task.into();
        let task_type = task.to_string();

        #[cfg(feature = "tracing")]
        Span::current().record("captcha.task_type", &task_type);

        #[cfg(feature = "tracing")]
        debug!(
            task_type = %task_type,
            timeout_secs = %self.config.timeout.as_secs_f64(),
            "Creating captcha task"
        );

        let start = Instant::now();

        // Create the task
        let outcome = self.provider.create_task(task).await.map_err(|e| {
            #[cfg(feature = "metrics")]
            ServiceMetrics::global().errors.add(
                1,
                &[
                    KeyValue::new("task_type", task_type.clone()),
                    KeyValue::new("operation", "create_task"),
                ],
            );
            ServiceError::from_provider(e)
        })?;

        #[cfg(feature = "metrics")]
        ServiceMetrics::global()
            .tasks_created
            .add(1, &[KeyValue::new("task_type", task_type.clone())]);

        // Handle immediate solution (e.g., ImageToText on Capsolver)
        let task_id = match outcome {
            TaskCreationOutcome::Ready { task_id, solution } => {
                let elapsed = start.elapsed();

                #[cfg(feature = "tracing")]
                {
                    Span::current().record("captcha.task_id", task_id.as_ref());
                    info!(
                        task_id = %task_id,
                        task_type = %task_type,
                        elapsed_secs = %elapsed.as_secs_f64(),
                        "Captcha solved immediately (no polling required)"
                    );
                }

                #[cfg(feature = "metrics")]
                {
                    ServiceMetrics::global()
                        .solutions_received
                        .add(1, &[KeyValue::new("task_type", task_type.clone())]);
                    ServiceMetrics::global().solve_time.record(
                        elapsed.as_secs_f64(),
                        &[
                            KeyValue::new("task_type", task_type.clone()),
                            KeyValue::new("outcome", "immediate"),
                        ],
                    );
                    ServiceMetrics::global().poll_counts.record(
                        0,
                        &[
                            KeyValue::new("task_type", task_type.clone()),
                            KeyValue::new("outcome", "immediate"),
                        ],
                    );
                }

                return Ok(solution);
            }
            TaskCreationOutcome::Pending(task_id) => task_id,
        };

        #[cfg(feature = "tracing")]
        {
            Span::current().record("captcha.task_id", task_id.as_ref());
            info!(
                task_id = %task_id,
                task_type = %task_type,
                "Captcha task created, polling for solution"
            );
        }

        // Poll for solution with timeout
        let timeout = self.config.timeout;
        let poll_interval = self.config.poll_interval;
        let start = Instant::now();
        let mut poll_count: u32 = 0;

        loop {
            // Check for cancellation first
            if cancel_token.is_cancelled() {
                let elapsed = start.elapsed();

                #[cfg(feature = "tracing")]
                info!(
                    task_id = %task_id,
                    elapsed_secs = %elapsed.as_secs_f64(),
                    poll_count = %poll_count,
                    "Cancellation requested"
                );

                #[cfg(feature = "metrics")]
                {
                    ServiceMetrics::global()
                        .cancellations
                        .add(1, &[KeyValue::new("task_type", task_type.clone())]);
                    ServiceMetrics::global().solve_time.record(
                        elapsed.as_secs_f64(),
                        &[
                            KeyValue::new("task_type", task_type.clone()),
                            KeyValue::new("outcome", "cancelled"),
                        ],
                    );
                    ServiceMetrics::global().poll_counts.record(
                        poll_count as u64,
                        &[
                            KeyValue::new("task_type", task_type.clone()),
                            KeyValue::new("outcome", "cancelled"),
                        ],
                    );
                }

                return Err(ServiceError::cancelled(elapsed, poll_count, task_id));
            }

            // Check for timeout
            let elapsed = start.elapsed();
            if elapsed >= timeout {
                #[cfg(feature = "tracing")]
                warn!(
                    task_id = %task_id,
                    timeout_secs = %timeout.as_secs_f64(),
                    elapsed_secs = %elapsed.as_secs_f64(),
                    poll_count = %poll_count,
                    "Captcha solution timeout"
                );

                #[cfg(feature = "metrics")]
                {
                    ServiceMetrics::global()
                        .timeouts
                        .add(1, &[KeyValue::new("task_type", task_type.clone())]);
                    ServiceMetrics::global().solve_time.record(
                        elapsed.as_secs_f64(),
                        &[
                            KeyValue::new("task_type", task_type.clone()),
                            KeyValue::new("outcome", "timeout"),
                        ],
                    );
                    ServiceMetrics::global().poll_counts.record(
                        poll_count as u64,
                        &[
                            KeyValue::new("task_type", task_type.clone()),
                            KeyValue::new("outcome", "timeout"),
                        ],
                    );
                }

                return Err(ServiceError::timeout(timeout, elapsed, poll_count, task_id));
            }

            poll_count += 1;

            match self.provider.get_task_result(&task_id).await {
                Ok(Some(solution)) => {
                    let elapsed = start.elapsed();

                    #[cfg(feature = "tracing")]
                    info!(
                        task_id = %task_id,
                        elapsed_secs = %elapsed.as_secs_f64(),
                        poll_count = %poll_count,
                        "Captcha solved successfully"
                    );

                    #[cfg(feature = "metrics")]
                    {
                        ServiceMetrics::global()
                            .solutions_received
                            .add(1, &[KeyValue::new("task_type", task_type.clone())]);
                        ServiceMetrics::global().solve_time.record(
                            elapsed.as_secs_f64(),
                            &[
                                KeyValue::new("task_type", task_type.clone()),
                                KeyValue::new("outcome", "success"),
                            ],
                        );
                        ServiceMetrics::global().poll_counts.record(
                            poll_count as u64,
                            &[
                                KeyValue::new("task_type", task_type.clone()),
                                KeyValue::new("outcome", "success"),
                            ],
                        );
                    }

                    return Ok(solution);
                }
                Ok(None) => {
                    // Solution not yet ready, continue polling
                    #[cfg(feature = "tracing")]
                    debug!(
                        task_id = %task_id,
                        poll_count = %poll_count,
                        elapsed_secs = %start.elapsed().as_secs_f64(),
                        "Solution not ready, continuing to poll"
                    );
                }
                Err(e) if !e.is_retryable() => {
                    // Permanent error - return immediately
                    let elapsed = start.elapsed();

                    #[cfg(feature = "tracing")]
                    error!(
                        task_id = %task_id,
                        error = %e,
                        is_retryable = %e.is_retryable(),
                        should_retry_operation = %e.should_retry_operation(),
                        elapsed_secs = %elapsed.as_secs_f64(),
                        poll_count = %poll_count,
                        "Permanent error while polling for solution"
                    );

                    #[cfg(feature = "metrics")]
                    {
                        ServiceMetrics::global().errors.add(
                            1,
                            &[
                                KeyValue::new("task_type", task_type.clone()),
                                KeyValue::new("operation", "get_task_result"),
                            ],
                        );
                        ServiceMetrics::global().solve_time.record(
                            elapsed.as_secs_f64(),
                            &[
                                KeyValue::new("task_type", task_type.clone()),
                                KeyValue::new("outcome", "error"),
                            ],
                        );
                        ServiceMetrics::global().poll_counts.record(
                            poll_count as u64,
                            &[
                                KeyValue::new("task_type", task_type.clone()),
                                KeyValue::new("outcome", "error"),
                            ],
                        );
                    }

                    return Err(ServiceError::from_provider(e));
                }
                Err(_e) => {
                    // Transient error - log and continue polling
                    #[cfg(feature = "tracing")]
                    warn!(
                        task_id = %task_id,
                        error = %_e,
                        poll_count = %poll_count,
                        "Transient error while polling, will retry"
                    );
                }
            }

            tokio::time::sleep(poll_interval).await;
        }
    }
}

/// Builder for CaptchaSolverService.
///
/// Provides a fluent API for constructing a captcha service with a provider
/// and custom configuration.
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::CaptchaSolverService;
/// use std::time::Duration;
///
/// let service = CaptchaSolverService::builder(provider)
///     .timeout(Duration::from_secs(180))
///     .poll_interval(Duration::from_secs(5))
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct CaptchaSolverServiceBuilder<P: Provider> {
    provider: P,
    config_builder: CaptchaSolverServiceConfigBuilder,
}

impl<P: Provider> CaptchaSolverServiceBuilder<P>
where
    P::Error: Debug + Display + RetryableError,
{
    /// Create a new builder with the given provider.
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            config_builder: CaptchaSolverServiceConfigBuilder::default(),
        }
    }

    /// Set the timeout for waiting for captcha solutions.
    ///
    /// Default: 120 seconds
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config_builder = self.config_builder.timeout(timeout);
        self
    }

    /// Set the polling interval when waiting for solutions.
    ///
    /// Default: 3 seconds
    pub fn poll_interval(mut self, interval: std::time::Duration) -> Self {
        self.config_builder = self.config_builder.poll_interval(interval);
        self
    }

    /// Set the full configuration.
    pub fn config(mut self, config: CaptchaSolverServiceConfig) -> Self {
        self.config_builder = CaptchaSolverServiceConfigBuilder {
            timeout: config.timeout,
            poll_interval: config.poll_interval,
        };
        self
    }

    /// Build the CaptchaSolverService.
    pub fn build(self) -> CaptchaSolverService<P> {
        CaptchaSolverService::with_config(self.provider, self.config_builder.build())
    }
}

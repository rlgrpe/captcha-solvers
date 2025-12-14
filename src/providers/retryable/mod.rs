//! Retryable provider wrapper.
//!
//! This module provides [`CaptchaRetryableProvider`], a wrapper that adds automatic
//! retry logic with exponential backoff to any provider.

use crate::errors::RetryableError;
use crate::providers::traits::Provider;
use crate::tasks::CaptchaTask;
use crate::utils::retry::RetryConfig;
use crate::utils::types::TaskId;
use backon::Retryable;
use std::fmt::Debug;

#[cfg(feature = "tracing")]
use tracing::debug;

/// Wrapper that adds automatic retry logic to any Provider.
///
/// This wrapper implements the same [`Provider`] trait but adds configurable
/// retry behavior based on the error's [`is_retryable()`](RetryableError::is_retryable) method.
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::{CaptchaRetryableProvider, RetryConfig, capsolver::CapsolverProvider};
/// use std::time::Duration;
///
/// let base_provider = CapsolverProvider::new("api_key")?;
///
/// // With default retry config
/// let provider = CaptchaRetryableProvider::new(base_provider.clone());
///
/// // With custom retry config
/// let custom_config = RetryConfig::default()
///     .with_max_retries(5)
///     .with_min_delay(Duration::from_millis(500));
/// let provider = CaptchaRetryableProvider::with_config(base_provider, custom_config);
///
/// // Use with service - all operations automatically retry on transient errors
/// let service = CaptchaSolverService::new(provider);
/// ```
#[derive(Debug, Clone)]
pub struct CaptchaRetryableProvider<P: Provider> {
    inner: P,
    retry_config: RetryConfig,
}

impl<P: Provider> CaptchaRetryableProvider<P> {
    /// Wrap a provider with default retry logic.
    ///
    /// Uses [`RetryConfig::default()`] which provides:
    /// - Initial delay: 1 second
    /// - Max delay: 30 seconds
    /// - Factor: 2x
    /// - Max retries: 3
    pub fn new(inner: P) -> Self {
        Self {
            inner,
            retry_config: RetryConfig::default(),
        }
    }

    /// Wrap a provider with custom retry configuration.
    pub fn with_config(inner: P, retry_config: RetryConfig) -> Self {
        Self {
            inner,
            retry_config,
        }
    }

    /// Get reference to the inner provider.
    pub fn inner(&self) -> &P {
        &self.inner
    }

    /// Get reference to the retry configuration.
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }
}

impl<P: Provider> Provider for CaptchaRetryableProvider<P>
where
    P::Error: Debug,
{
    type Solution = P::Solution;
    type Error = P::Error;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "captcha.provider.create_task.retry",
            skip_all,
            fields(captcha.task_type)
        )
    )]
    async fn create_task(&self, task: CaptchaTask) -> Result<TaskId, Self::Error> {
        #[cfg(feature = "tracing")]
        tracing::Span::current().record("captcha.task_type", task.to_string());

        let inner = self.inner.clone();
        let task_clone = task.clone();
        (|| async { inner.create_task(task_clone.clone()).await })
            .retry(self.retry_config.build_strategy())
            .when(|err: &Self::Error| err.is_retryable())
            .notify(|err, duration| {
                let _ = (err, duration);
                #[cfg(feature = "tracing")]
                debug!(
                    error = ?err,
                    captcha.task_type = %task,
                    retry_after_secs = %duration.as_secs_f64(),
                    "Retrying create_task after transient error"
                );
            })
            .await
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "captcha.provider.get_task_result.retry",
            skip_all,
            fields(captcha.task_id = %task_id)
        )
    )]
    async fn get_task_result(
        &self,
        task_id: &TaskId,
    ) -> Result<Option<Self::Solution>, Self::Error> {
        let inner = self.inner.clone();
        let task_id = task_id.clone();
        (|| async { inner.get_task_result(&task_id).await })
            .retry(self.retry_config.build_strategy())
            .when(|err: &Self::Error| err.is_retryable())
            .notify(|err, duration| {
                let _ = (err, duration);
                #[cfg(feature = "tracing")]
                debug!(
                    error = ?err,
                    captcha.task_id = %task_id,
                    retry_after_secs = %duration.as_secs_f64(),
                    "Retrying get_task_result after transient error"
                );
            })
            .await
    }
}

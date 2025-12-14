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
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "tracing")]
use tracing::debug;

/// Callback type for retry notifications.
///
/// This callback is invoked each time a retry is attempted.
/// The callback receives the error that caused the retry and the duration
/// until the next retry attempt.
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::CaptchaRetryableProvider;
///
/// let provider = CaptchaRetryableProvider::new(base_provider)
///     .with_on_retry(|error, duration| {
///         println!("Retrying after {:?} due to: {}", duration, error);
///     });
/// ```
pub type OnRetryCallback<E> = Arc<dyn Fn(&E, Duration) + Send + Sync>;

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
/// let provider = CaptchaRetryableProvider::with_config(base_provider.clone(), custom_config);
///
/// // With retry callback
/// let provider = CaptchaRetryableProvider::new(base_provider)
///     .with_on_retry(|error, duration| {
///         println!("Retrying after {:?} due to: {}", duration, error);
///     });
///
/// // Use with service - all operations automatically retry on transient errors
/// let service = CaptchaSolverService::new(provider);
/// ```
pub struct CaptchaRetryableProvider<P: Provider> {
    inner: Arc<P>,
    retry_config: RetryConfig,
    on_retry: Option<OnRetryCallback<P::Error>>,
}

impl<P: Provider> Clone for CaptchaRetryableProvider<P> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            retry_config: self.retry_config.clone(),
            on_retry: self.on_retry.clone(),
        }
    }
}

impl<P: Provider + Debug> Debug for CaptchaRetryableProvider<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CaptchaRetryableProvider")
            .field("inner", &self.inner)
            .field("retry_config", &self.retry_config)
            .field("on_retry", &self.on_retry.as_ref().map(|_| "..."))
            .finish()
    }
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
            inner: Arc::new(inner),
            retry_config: RetryConfig::default(),
            on_retry: None,
        }
    }

    /// Wrap a provider with custom retry configuration.
    pub fn with_config(inner: P, retry_config: RetryConfig) -> Self {
        Self {
            inner: Arc::new(inner),
            retry_config,
            on_retry: None,
        }
    }

    /// Set a callback to be invoked on each retry attempt.
    ///
    /// The callback receives the error that caused the retry and the duration
    /// until the next retry attempt.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let provider = CaptchaRetryableProvider::new(base_provider)
    ///     .with_on_retry(|error, duration| {
    ///         println!("Retrying after {:?} due to: {}", duration, error);
    ///     });
    /// ```
    pub fn with_on_retry<F>(mut self, callback: F) -> Self
    where
        F: Fn(&P::Error, Duration) + Send + Sync + 'static,
    {
        self.on_retry = Some(Arc::new(callback));
        self
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

        let inner = Arc::clone(&self.inner);
        let task_for_notify = task.clone();
        let on_retry = self.on_retry.clone();
        (|| {
            let inner = Arc::clone(&inner);
            let task = task.clone();
            async move { inner.create_task(task).await }
        })
        .retry(self.retry_config.build_strategy())
        .when(|err: &Self::Error| err.is_retryable())
        .notify(move |err, duration| {
            // Call user callback if set
            if let Some(ref callback) = on_retry {
                callback(err, duration);
            }

            #[cfg(feature = "tracing")]
            debug!(
                error = ?err,
                captcha.task_type = %task_for_notify,
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
        let inner = Arc::clone(&self.inner);
        let task_id_owned = task_id.clone();
        let task_id_for_notify = task_id.clone();
        let on_retry = self.on_retry.clone();
        (|| {
            let inner = Arc::clone(&inner);
            let task_id = task_id_owned.clone();
            async move { inner.get_task_result(&task_id).await }
        })
        .retry(self.retry_config.build_strategy())
        .when(|err: &Self::Error| err.is_retryable())
        .notify(move |err, duration| {
            // Call user callback if set
            if let Some(ref callback) = on_retry {
                callback(err, duration);
            }

            #[cfg(feature = "tracing")]
            debug!(
                error = ?err,
                captcha.task_id = %task_id_for_notify,
                retry_after_secs = %duration.as_secs_f64(),
                "Retrying get_task_result after transient error"
            );
        })
        .await
    }
}

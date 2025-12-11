//! RuCaptcha provider implementation.

use super::client::RucaptchaClient;
use super::errors::RucaptchaError;
use super::types::{RucaptchaSolution, RucaptchaTask};
use crate::provider::Provider;
use crate::types::TaskId;

/// RuCaptcha provider implementation
///
/// This wraps the [`RucaptchaClient`] and implements the generic [`Provider`] trait,
/// allowing it to be used with [`CaptchaSolverService`](crate::CaptchaSolverService)
/// and [`RetryableProvider`](crate::RetryableProvider).
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::providers::rucaptcha::{RucaptchaClient, RucaptchaProvider, RucaptchaTask};
/// use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait};
/// use std::time::Duration;
///
/// let client = RucaptchaClient::new("api_key")?;
/// let provider = RucaptchaProvider::new(client);
/// let service = CaptchaSolverService::with_provider(provider);
///
/// let task = RucaptchaTask::turnstile("https://example.com", "site_key");
/// let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;
/// ```
#[derive(Debug, Clone)]
pub struct RucaptchaProvider {
    client: RucaptchaClient,
}

impl RucaptchaProvider {
    /// Create a new RuCaptcha provider
    ///
    /// # Arguments
    /// * `client` - The RuCaptcha HTTP client
    pub fn new(client: RucaptchaClient) -> Self {
        Self { client }
    }

    /// Get reference to the inner client
    pub fn client(&self) -> &RucaptchaClient {
        &self.client
    }

    /// Get mutable reference to the inner client
    pub fn client_mut(&mut self) -> &mut RucaptchaClient {
        &mut self.client
    }

    /// Consume the provider and return the inner client
    pub fn into_client(self) -> RucaptchaClient {
        self.client
    }
}

impl Provider for RucaptchaProvider {
    type Task = RucaptchaTask;
    type Solution = RucaptchaSolution;
    type Error = RucaptchaError;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(name = "RucaptchaProvider::create_task", skip_all)
    )]
    async fn create_task(&self, task: Self::Task) -> Result<TaskId, Self::Error> {
        self.client.create_task(task).await
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "RucaptchaProvider::get_task_result",
            skip_all,
            fields(task_id = %task_id)
        )
    )]
    async fn get_task_result(
        &self,
        task_id: &TaskId,
    ) -> Result<Option<Self::Solution>, Self::Error> {
        self.client.get_task_result(task_id).await
    }
}
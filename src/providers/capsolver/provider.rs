use super::client::CapsolverClient;
use super::errors::CapsolverError;
use super::types::{CapsolverSolution, CapsolverTask};
use crate::provider::Provider;
use crate::types::TaskId;

/// Capsolver provider implementation
///
/// This wraps the [`CapsolverClient`] and implements the generic [`Provider`] trait,
/// allowing it to be used with [`CaptchaSolverService`](crate::CaptchaSolverService)
/// and [`RetryableProvider`](crate::RetryableProvider).
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::providers::capsolver::{CapsolverClient, CapsolverProvider, CapsolverTask};
/// use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait};
/// use std::time::Duration;
///
/// let client = CapsolverClient::new("api_key")?;
/// let provider = CapsolverProvider::new(client);
/// let service = CaptchaSolverService::with_provider(provider);
///
/// let task = CapsolverTask::turnstile("https://example.com", "site_key");
/// let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;
/// ```
#[derive(Debug, Clone)]
pub struct CapsolverProvider {
    client: CapsolverClient,
}

impl CapsolverProvider {
    /// Create a new Capsolver provider
    ///
    /// # Arguments
    /// * `client` - The Capsolver HTTP client
    pub fn new(client: CapsolverClient) -> Self {
        Self { client }
    }

    /// Get reference to the inner client
    pub fn client(&self) -> &CapsolverClient {
        &self.client
    }

    /// Get mutable reference to the inner client
    pub fn client_mut(&mut self) -> &mut CapsolverClient {
        &mut self.client
    }

    /// Consume the provider and return the inner client
    pub fn into_client(self) -> CapsolverClient {
        self.client
    }
}

impl Provider for CapsolverProvider {
    type Task = CapsolverTask;
    type Solution = CapsolverSolution;
    type Error = CapsolverError;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(name = "CapsolverProvider::create_task", skip_all)
    )]
    async fn create_task(&self, task: Self::Task) -> Result<TaskId, Self::Error> {
        self.client.create_task(task).await
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "CapsolverProvider::get_task_result",
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
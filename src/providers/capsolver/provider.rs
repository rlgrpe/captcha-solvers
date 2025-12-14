//! Capsolver provider implementation.

use super::errors::{CapsolverError, Result};
use super::response::CapsolverResponse;
use super::types::{
    CapsolverSolution, CapsolverTask, CreateTaskData, CreateTaskRequest, GetTaskData,
    GetTaskResultRequest,
};
use crate::providers::traits::Provider;
use crate::tasks::CaptchaTask;
use crate::utils::types::TaskId;
use reqwest::Url;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[cfg(feature = "tracing")]
use opentelemetry::trace::Status;
#[cfg(feature = "tracing")]
use tracing::Span;
#[cfg(feature = "tracing")]
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Default Capsolver API URL
pub const DEFAULT_API_URL: &str = "https://api.capsolver.com";

/// API endpoint paths
const CREATE_TASK_PATH: &str = "createTask";
const GET_TASK_RESULT_PATH: &str = "getTaskResult";

/// Capsolver provider implementation
///
/// This provider handles all communication with the Capsolver API,
/// including task creation and solution polling.
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::{
///     CaptchaSolverService, CaptchaSolverServiceTrait,
///     ReCaptchaV2, Turnstile,
///     providers::capsolver::CapsolverProvider,
/// };
/// use std::time::Duration;
///
/// // Create provider directly with API key
/// let provider = CapsolverProvider::new("api_key")?;
/// let service = CaptchaSolverService::with_provider(provider);
///
/// // Use shared task types
/// let task = ReCaptchaV2::new("https://example.com", "site_key")
///     .invisible()
///     .enterprise();
///
/// let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;
/// println!("Token: {}", solution.into_recaptcha().token());
/// ```
#[derive(Clone)]
pub struct CapsolverProvider {
    http_client: ClientWithMiddleware,
    api_key: SecretString,
    url: Url,
}

impl Debug for CapsolverProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapsolverProvider")
            .field("url", &self.url)
            .field("api_key", &"[REDACTED]")
            .finish()
    }
}

/// Builder for configuring a [`CapsolverProvider`]
///
/// Provides a fluent API for constructing providers with custom settings.
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::providers::capsolver::CapsolverProvider;
/// use url::Url;
///
/// let provider = CapsolverProvider::builder("your-api-key")
///     .url(Url::parse("https://custom-api.example.com").unwrap())
///     .http_client(custom_client)
///     .build()?;
/// ```
pub struct CapsolverProviderBuilder {
    api_key: String,
    url: Option<Url>,
    http_client: Option<ClientWithMiddleware>,
}

impl CapsolverProviderBuilder {
    /// Create a new builder with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            url: None,
            http_client: None,
        }
    }

    /// Set a custom API URL
    ///
    /// Default: `https://api.capsolver.com`
    pub fn url(mut self, url: Url) -> Self {
        self.url = Some(url);
        self
    }

    /// Set a custom HTTP client with middleware
    ///
    /// Use this when you need custom middleware (e.g., tracing, retry, rate limiting).
    pub fn http_client(mut self, client: ClientWithMiddleware) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Build the [`CapsolverProvider`]
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be built.
    pub fn build(self) -> Result<CapsolverProvider> {
        let url = self
            .url
            .unwrap_or_else(|| Url::parse(DEFAULT_API_URL).expect("Invalid default URL"));

        let http_client = match self.http_client {
            Some(client) => client,
            None => {
                let client = reqwest::Client::builder()
                    .build()
                    .map_err(CapsolverError::BuildHttpClient)?;
                ClientBuilder::new(client).build()
            }
        };

        Ok(CapsolverProvider {
            http_client,
            api_key: SecretString::from(self.api_key),
            url,
        })
    }
}

impl CapsolverProvider {
    /// Create a new Capsolver provider with the default API URL
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Capsolver API key
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let provider = CapsolverProvider::new("your_api_key")?;
    /// ```
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::builder(api_key).build()
    }

    /// Create a new Capsolver provider with a custom URL
    ///
    /// # Arguments
    ///
    /// * `url` - Base URL for the Capsolver API
    /// * `api_key` - Your Capsolver API key
    pub fn with_url(url: Url, api_key: impl Into<String>) -> Result<Self> {
        Self::builder(api_key).url(url).build()
    }

    /// Create a builder for configuring the provider
    ///
    /// Use this for advanced configuration options like custom HTTP clients.
    pub fn builder(api_key: impl Into<String>) -> CapsolverProviderBuilder {
        CapsolverProviderBuilder::new(api_key)
    }

    /// Get the base URL
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get the API key (exposed for request building).
    fn api_key(&self) -> &str {
        self.api_key.expose_secret()
    }

    /// Send a POST request to the API.
    async fn post<Req: serde::Serialize, Res: DeserializeOwned>(
        &self,
        path: &str,
        request: &Req,
    ) -> Result<Res> {
        let mut url = self.url.clone();
        url.set_path(path);

        let response = self
            .http_client
            .post(url)
            .json(request)
            .send()
            .await
            .map_err(CapsolverError::HttpRequest)?;

        response.json().await.map_err(CapsolverError::ParseResponse)
    }

    /// Create a captcha solving task (internal)
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(name = "CapsolverProvider::create_task_internal", skip_all)
    )]
    async fn create_task_internal(&self, task: CapsolverTask) -> Result<TaskId> {
        let request = CreateTaskRequest {
            client_key: self.api_key(),
            task: &task,
        };

        let response: CapsolverResponse<CreateTaskData> =
            self.post(CREATE_TASK_PATH, &request).await?;

        let data = response.into_result().map_err(CapsolverError::Api)?;
        let task_id = TaskId::from(data.task_id);

        #[cfg(feature = "tracing")]
        {
            Span::current()
                .record("task_id", task_id.as_ref())
                .set_status(Status::Ok);
        }

        Ok(task_id)
    }

    /// Get the result of a captcha task (internal)
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "CapsolverProvider::get_task_result_internal",
            skip_all,
            fields(task_id = %task_id)
        )
    )]
    async fn get_task_result_internal<T: DeserializeOwned + Debug>(
        &self,
        task_id: &TaskId,
    ) -> Result<Option<T>> {
        let request = GetTaskResultRequest {
            client_key: self.api_key(),
            task_id: task_id.as_ref(),
        };

        let response: CapsolverResponse<GetTaskData<T>> =
            self.post(GET_TASK_RESULT_PATH, &request).await?;

        let data = response.into_result().map_err(CapsolverError::Api)?;

        #[cfg(feature = "tracing")]
        if data.solution.is_some() {
            Span::current().set_status(Status::Ok);
        }

        Ok(data.solution)
    }
}

impl Provider for CapsolverProvider {
    type Solution = CapsolverSolution;
    type Error = CapsolverError;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(name = "CapsolverProvider::create_task", skip_all)
    )]
    async fn create_task(&self, task: CaptchaTask) -> Result<TaskId> {
        // Convert unified task to provider-specific format
        let internal_task: CapsolverTask = task.into();
        self.create_task_internal(internal_task).await
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "CapsolverProvider::get_task_result",
            skip_all,
            fields(task_id = %task_id)
        )
    )]
    async fn get_task_result(&self, task_id: &TaskId) -> Result<Option<Self::Solution>> {
        self.get_task_result_internal(task_id).await
    }
}

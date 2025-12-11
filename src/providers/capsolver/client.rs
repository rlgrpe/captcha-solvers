//! Capsolver HTTP client for making API requests.

use super::errors::{CapsolverError, Result};
use super::response::CapsolverResponse;
use super::types::{
    CapsolverTask, CreateTaskData, CreateTaskRequest, GetTaskData, GetTaskResultRequest,
};
use crate::types::TaskId;
use reqwest::Url;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

/// Default Capsolver API URL
pub const DEFAULT_API_URL: &str = "https://api.capsolver.com";

/// API endpoint paths
const CREATE_TASK_PATH: &str = "createTask";
const GET_TASK_RESULT_PATH: &str = "getTaskResult";

#[cfg(feature = "tracing")]
use opentelemetry::trace::Status;
#[cfg(feature = "tracing")]
use tracing::Span;
#[cfg(feature = "tracing")]
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Capsolver HTTP client
///
/// Low-level client for interacting with the Capsolver API.
/// Handles HTTP requests, authentication, and response parsing.
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::providers::capsolver::{CapsolverClient, CapsolverTask};
///
/// // Create client with default URL
/// let client = CapsolverClient::new("your_api_key")?;
///
/// // Create a task
/// let task = CapsolverTask::turnstile("https://example.com", "site_key");
/// let task_id = client.create_task(task).await?;
///
/// // Poll for result
/// let solution = client.get_task_result(&task_id).await?;
/// ```
#[derive(Clone)]
pub struct CapsolverClient {
    http_client: ClientWithMiddleware,
    api_key: SecretString,
    pub(crate) url: Url,
}

impl Debug for CapsolverClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapsolverClient")
            .field("url", &self.url)
            .field("api_key", &"[REDACTED]")
            .finish()
    }
}

/// Builder for configuring a [`CapsolverClient`]
///
/// Provides a fluent API for constructing clients with custom settings.
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::providers::capsolver::CapsolverClient;
/// use url::Url;
///
/// let client = CapsolverClient::builder("your-api-key")
///     .url(Url::parse("https://custom-api.example.com").unwrap())
///     .http_client(custom_client)
///     .build()?;
/// ```
pub struct CapsolverClientBuilder {
    api_key: String,
    url: Option<Url>,
    http_client: Option<ClientWithMiddleware>,
}

impl CapsolverClientBuilder {
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

    /// Build the [`CapsolverClient`]
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be built.
    pub fn build(self) -> Result<CapsolverClient> {
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

        Ok(CapsolverClient {
            http_client,
            api_key: SecretString::from(self.api_key),
            url,
        })
    }
}

impl CapsolverClient {
    /// Create a new Capsolver client with the default API URL
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Capsolver API key
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = CapsolverClient::new("your_api_key")?;
    /// ```
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::builder(api_key).build()
    }

    /// Create a new Capsolver client with a custom URL
    ///
    /// # Arguments
    ///
    /// * `url` - Base URL for the Capsolver API
    /// * `api_key` - Your Capsolver API key
    pub fn with_url(url: Url, api_key: impl Into<String>) -> Result<Self> {
        Self::builder(api_key).url(url).build()
    }

    /// Create a builder for configuring the client
    ///
    /// Use this for advanced configuration options like custom HTTP clients.
    pub fn builder(api_key: impl Into<String>) -> CapsolverClientBuilder {
        CapsolverClientBuilder::new(api_key)
    }

    /// Send a POST request to the Capsolver API
    async fn post<Req: Serialize, Res: DeserializeOwned>(
        &self,
        path: &str,
        request: &Req,
    ) -> Result<Res> {
        let mut url = self.url.clone();
        url.set_path(path);

        let response = self.http_client.post(url).json(request).send().await?;

        response
            .json()
            .await
            .map_err(CapsolverError::ParseResponse)
    }

    /// Create a captcha solving task
    ///
    /// Submits a new task to the Capsolver API and returns the task ID.
    /// Use [`get_task_result`](Self::get_task_result) to poll for the solution.
    ///
    /// # Arguments
    ///
    /// * `task` - The captcha task to solve
    ///
    /// # Returns
    ///
    /// The task ID that can be used to retrieve the result.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(name = "CapsolverClient::create_task", skip_all)
    )]
    pub async fn create_task(&self, task: CapsolverTask) -> Result<TaskId> {
        let request = CreateTaskRequest {
            client_key: self.api_key.expose_secret(),
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

    /// Get the result of a captcha task
    ///
    /// Polls the Capsolver API for the task result.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task ID returned from [`create_task`](Self::create_task)
    ///
    /// # Returns
    ///
    /// - `Ok(Some(solution))` if the task is complete
    /// - `Ok(None)` if the task is still processing
    /// - `Err(...)` if there was an error
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "CapsolverClient::get_task_result",
            skip_all,
            fields(task_id = %task_id)
        )
    )]
    pub async fn get_task_result<T: DeserializeOwned + Debug>(
        &self,
        task_id: &TaskId,
    ) -> Result<Option<T>> {
        let request = GetTaskResultRequest {
            client_key: self.api_key.expose_secret(),
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
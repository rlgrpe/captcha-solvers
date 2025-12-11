//! RuCaptcha HTTP client for making API requests.

use super::errors::{RucaptchaError, Result};
use super::response::RucaptchaResponse;
use super::types::{
    CreateTaskData, CreateTaskRequest, GetTaskData, GetTaskResultRequest, RucaptchaTask,
};
use crate::types::TaskId;
use reqwest::Url;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

/// Default RuCaptcha API URL
pub const DEFAULT_API_URL: &str = "https://api.rucaptcha.com";

/// API endpoint paths
const CREATE_TASK_PATH: &str = "createTask";
const GET_TASK_RESULT_PATH: &str = "getTaskResult";

#[cfg(feature = "tracing")]
use opentelemetry::trace::Status;
#[cfg(feature = "tracing")]
use tracing::Span;
#[cfg(feature = "tracing")]
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// RuCaptcha HTTP client
///
/// Low-level client for interacting with the RuCaptcha API.
/// Handles HTTP requests, authentication, and response parsing.
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::providers::rucaptcha::{RucaptchaClient, RucaptchaTask};
///
/// // Create client with default URL
/// let client = RucaptchaClient::new("your_api_key")?;
///
/// // Create a task
/// let task = RucaptchaTask::turnstile("https://example.com", "site_key");
/// let task_id = client.create_task(task).await?;
///
/// // Poll for result
/// let solution = client.get_task_result(&task_id).await?;
/// ```
#[derive(Clone)]
pub struct RucaptchaClient {
    http_client: ClientWithMiddleware,
    api_key: SecretString,
    pub(crate) url: Url,
}

impl Debug for RucaptchaClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RucaptchaClient")
            .field("url", &self.url)
            .field("api_key", &"[REDACTED]")
            .finish()
    }
}

/// Builder for configuring a [`RucaptchaClient`]
///
/// Provides a fluent API for constructing clients with custom settings.
///
/// # Example
///
/// ```rust,ignore
/// use captcha_solvers::providers::rucaptcha::RucaptchaClient;
/// use url::Url;
///
/// let client = RucaptchaClient::builder("your-api-key")
///     .url(Url::parse("https://custom-api.example.com").unwrap())
///     .http_client(custom_client)
///     .build()?;
/// ```
pub struct RucaptchaClientBuilder {
    api_key: String,
    url: Option<Url>,
    http_client: Option<ClientWithMiddleware>,
}

impl RucaptchaClientBuilder {
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
    /// Default: `https://api.rucaptcha.com`
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

    /// Build the [`RucaptchaClient`]
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be built.
    pub fn build(self) -> Result<RucaptchaClient> {
        let url = self
            .url
            .unwrap_or_else(|| Url::parse(DEFAULT_API_URL).expect("Invalid default URL"));

        let http_client = match self.http_client {
            Some(client) => client,
            None => {
                let client = reqwest::Client::builder()
                    .build()
                    .map_err(RucaptchaError::BuildHttpClient)?;
                ClientBuilder::new(client).build()
            }
        };

        Ok(RucaptchaClient {
            http_client,
            api_key: SecretString::from(self.api_key),
            url,
        })
    }
}

impl RucaptchaClient {
    /// Create a new RuCaptcha client with the default API URL
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your RuCaptcha API key
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = RucaptchaClient::new("your_api_key")?;
    /// ```
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::builder(api_key).build()
    }

    /// Create a new RuCaptcha client with a custom URL
    ///
    /// # Arguments
    ///
    /// * `url` - Base URL for the RuCaptcha API
    /// * `api_key` - Your RuCaptcha API key
    pub fn with_url(url: Url, api_key: impl Into<String>) -> Result<Self> {
        Self::builder(api_key).url(url).build()
    }

    /// Create a builder for configuring the client
    ///
    /// Use this for advanced configuration options like custom HTTP clients.
    pub fn builder(api_key: impl Into<String>) -> RucaptchaClientBuilder {
        RucaptchaClientBuilder::new(api_key)
    }

    /// Send a POST request to the RuCaptcha API
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
            .map_err(RucaptchaError::ParseResponse)
    }

    /// Create a captcha solving task
    ///
    /// Submits a new task to the RuCaptcha API and returns the task ID.
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
        tracing::instrument(name = "RucaptchaClient::create_task", skip_all)
    )]
    pub async fn create_task(&self, task: RucaptchaTask) -> Result<TaskId> {
        let request = CreateTaskRequest {
            client_key: self.api_key.expose_secret(),
            task: &task,
        };

        let response: RucaptchaResponse<CreateTaskData> =
            self.post(CREATE_TASK_PATH, &request).await?;

        let data = response.into_result().map_err(RucaptchaError::Api)?;
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
    /// Polls the RuCaptcha API for the task result.
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
            name = "RucaptchaClient::get_task_result",
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

        let response: RucaptchaResponse<GetTaskData<T>> =
            self.post(GET_TASK_RESULT_PATH, &request).await?;

        let data = response.into_result().map_err(RucaptchaError::Api)?;

        #[cfg(feature = "tracing")]
        if data.solution.is_some() {
            Span::current().set_status(Status::Ok);
        }

        Ok(data.solution)
    }
}
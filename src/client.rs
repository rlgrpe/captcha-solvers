//! Generic API client infrastructure for captcha solving providers.
//!
//! This module provides reusable client components that can be shared
//! across different captcha solving providers.

use reqwest::Url;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

#[cfg(feature = "tracing")]
use opentelemetry::trace::Status;
#[cfg(feature = "tracing")]
use tracing::Span;
#[cfg(feature = "tracing")]
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::types::TaskId;

/// Trait for provider-specific API configuration
///
/// Implement this trait to define how your provider's API works,
/// including URL patterns, request/response types, and error handling.
pub trait ApiClientConfig: Clone + Send + Sync {
    /// The task type for this provider
    type Task: Clone + Send + Sync + Serialize;

    /// The solution type for this provider
    type Solution: Send + Sync + DeserializeOwned + Debug;

    /// The API error type for this provider
    type ApiError: std::error::Error + Send + Sync + 'static;

    /// The client error type for this provider
    type Error: std::error::Error + Send + Sync + 'static;

    /// Default API URL for this provider
    fn default_url() -> &'static str;

    /// Path for creating tasks (e.g., "createTask")
    fn create_task_path() -> &'static str;

    /// Path for getting task results (e.g., "getTaskResult")
    fn get_result_path() -> &'static str;

    /// Create error from HTTP client build failure
    fn build_http_client_error(err: reqwest::Error) -> Self::Error;

    /// Create error from response parsing failure
    fn parse_response_error(err: reqwest::Error) -> Self::Error;

    /// Create error from API error response
    fn api_error(err: Self::ApiError) -> Self::Error;
}

/// Generic API client that can be used with any provider
///
/// This struct handles the common HTTP client logic and can be
/// configured for different providers through the `ApiClientConfig` trait.
#[derive(Clone)]
pub struct GenericApiClient<C: ApiClientConfig> {
    http_client: ClientWithMiddleware,
    api_key: SecretString,
    pub(crate) url: Url,
    _config: std::marker::PhantomData<C>,
}

impl<C: ApiClientConfig> Debug for GenericApiClient<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericApiClient")
            .field("url", &self.url)
            .field("api_key", &"[REDACTED]")
            .finish()
    }
}

/// Builder for configuring a generic API client
pub struct GenericApiClientBuilder<C: ApiClientConfig> {
    api_key: String,
    url: Option<Url>,
    http_client: Option<ClientWithMiddleware>,
    _config: std::marker::PhantomData<C>,
}

impl<C: ApiClientConfig> GenericApiClientBuilder<C> {
    /// Create a new builder with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            url: None,
            http_client: None,
            _config: std::marker::PhantomData,
        }
    }

    /// Set a custom API URL
    pub fn url(mut self, url: Url) -> Self {
        self.url = Some(url);
        self
    }

    /// Set a custom HTTP client with middleware
    pub fn http_client(mut self, client: ClientWithMiddleware) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Build the client
    pub fn build(self) -> Result<GenericApiClient<C>, C::Error> {
        let url = self
            .url
            .unwrap_or_else(|| Url::parse(C::default_url()).expect("Invalid default URL"));

        let http_client = match self.http_client {
            Some(client) => client,
            None => {
                let client = reqwest::Client::builder()
                    .build()
                    .map_err(C::build_http_client_error)?;
                ClientBuilder::new(client).build()
            }
        };

        Ok(GenericApiClient {
            http_client,
            api_key: SecretString::from(self.api_key),
            url,
            _config: std::marker::PhantomData,
        })
    }
}

impl<C: ApiClientConfig> GenericApiClient<C> {
    /// Create a new client with the default API URL
    pub fn new(api_key: impl Into<String>) -> Result<Self, C::Error> {
        Self::builder(api_key).build()
    }

    /// Create a new client with a custom URL
    pub fn with_url(url: Url, api_key: impl Into<String>) -> Result<Self, C::Error> {
        Self::builder(api_key).url(url).build()
    }

    /// Create a builder for configuring the client
    pub fn builder(api_key: impl Into<String>) -> GenericApiClientBuilder<C> {
        GenericApiClientBuilder::new(api_key)
    }

    /// Get the API key (exposed for request building)
    pub fn api_key(&self) -> &str {
        self.api_key.expose_secret()
    }

    /// Get the base URL
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Send a POST request to the API
    pub async fn post<Req: Serialize, Res: DeserializeOwned>(
        &self,
        path: &str,
        request: &Req,
    ) -> Result<Res, reqwest_middleware::Error> {
        let mut url = self.url.clone();
        url.set_path(path);

        let response = self.http_client.post(url).json(request).send().await?;

        response.json().await.map_err(reqwest_middleware::Error::from)
    }
}

/// Trait for creating task requests
pub trait CreateTaskRequest<'a>: Serialize {
    type Task;

    fn new(client_key: &'a str, task: &'a Self::Task) -> Self;
}

/// Trait for creating get result requests
pub trait GetTaskResultRequest<'a>: Serialize {
    fn new(client_key: &'a str, task_id: &'a str) -> Self;
}

/// Trait for parsing create task responses
pub trait CreateTaskResponse {
    type Error;

    fn into_task_id(self) -> Result<TaskId, Self::Error>;
}

/// Trait for parsing get result responses
pub trait GetTaskResultResponse<T> {
    type Error;

    fn into_solution(self) -> Result<Option<T>, Self::Error>;
}

/// Helper macro to record tracing span status on success
#[cfg(feature = "tracing")]
pub fn record_success() {
    Span::current().set_status(Status::Ok);
}

/// Helper macro to record task_id in tracing span
#[cfg(feature = "tracing")]
pub fn record_task_id(task_id: &TaskId) {
    Span::current()
        .record("task_id", task_id.as_ref())
        .set_status(Status::Ok);
}

#[cfg(not(feature = "tracing"))]
pub fn record_success() {}

#[cfg(not(feature = "tracing"))]
pub fn record_task_id(_task_id: &TaskId) {}
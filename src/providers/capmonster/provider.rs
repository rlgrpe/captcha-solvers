//! CapMonster provider implementation.

use super::errors::{CapmonsterError, Result};
use super::response::CapmonsterResponse;
use super::types::{
    CapmonsterSolution, CapmonsterTask, CreateTaskData, CreateTaskRequest, GetTaskData,
    GetTaskResultRequest,
};
use crate::providers::traits::{Provider, TaskCreationOutcome};
use crate::tasks::CaptchaTask;
use crate::utils::types::TaskId;
use reqwest::Url;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[cfg(feature = "tracing")]
use crate::utils::error_chain::ErrorChain;
#[cfg(feature = "tracing")]
use crate::utils::span_status::{set_span_error, set_span_ok};
#[cfg(feature = "tracing")]
use tracing::Span;

/// Default CapMonster API URL.
pub const DEFAULT_API_URL: &str = "https://api.capmonster.cloud";

/// API endpoint paths.
const CREATE_TASK_PATH: &str = "createTask";
const GET_TASK_RESULT_PATH: &str = "getTaskResult";

/// CapMonster provider implementation.
#[derive(Clone)]
pub struct CapmonsterProvider {
    http_client: ClientWithMiddleware,
    api_key: SecretString,
    url: Url,
}

impl Debug for CapmonsterProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapmonsterProvider")
            .field("url", &self.url)
            .field("api_key", &crate::utils::REDACTED)
            .finish()
    }
}

/// Builder for configuring a [`CapmonsterProvider`].
pub struct CapmonsterProviderBuilder {
    api_key: String,
    url: Option<Url>,
    http_client: Option<ClientWithMiddleware>,
}

impl CapmonsterProviderBuilder {
    /// Create a new builder with API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            url: None,
            http_client: None,
        }
    }

    /// Set a custom API URL.
    pub fn url(mut self, url: Url) -> Self {
        self.url = Some(url);
        self
    }

    /// Set custom middleware HTTP client.
    pub fn http_client(mut self, client: ClientWithMiddleware) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Build provider instance.
    pub fn build(self) -> Result<CapmonsterProvider> {
        let url = self
            .url
            .unwrap_or_else(|| Url::parse(DEFAULT_API_URL).expect("Invalid default URL"));

        let http_client = match self.http_client {
            Some(client) => client,
            None => {
                let client = reqwest::Client::builder()
                    .build()
                    .map_err(CapmonsterError::BuildHttpClient)?;
                ClientBuilder::new(client).build()
            }
        };

        Ok(CapmonsterProvider {
            http_client,
            api_key: SecretString::from(self.api_key),
            url,
        })
    }
}

impl CapmonsterProvider {
    /// Create provider with default API URL.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::builder(api_key).build()
    }

    /// Create provider with custom URL.
    pub fn with_url(url: Url, api_key: impl Into<String>) -> Result<Self> {
        Self::builder(api_key).url(url).build()
    }

    /// Create provider builder.
    pub fn builder(api_key: impl Into<String>) -> CapmonsterProviderBuilder {
        CapmonsterProviderBuilder::new(api_key)
    }

    /// Get base URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get API key.
    fn api_key(&self) -> &str {
        self.api_key.expose_secret()
    }

    /// Send POST request to provider API.
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
            .map_err(CapmonsterError::HttpRequest)?;

        response
            .json()
            .await
            .map_err(CapmonsterError::ParseResponse)
    }

    fn validate_task(task: &CapmonsterTask) -> Result<()> {
        use CapmonsterTask::*;

        match task {
            RecaptchaV3TaskProxyless {
                min_score: Some(score),
                ..
            }
            | RecaptchaV3EnterpriseTask {
                min_score: Some(score),
                ..
            } if !(0.1..=0.9).contains(score) => {
                return Err(CapmonsterError::InvalidTaskData(
                    "minScore must be in range 0.1..=0.9".to_string(),
                ));
            }
            TurnstileTask {
                cloudflare_task_type: Some(task_type),
                page_action,
                data,
                page_data,
                user_agent,
                html_page_base64,
                proxy,
                ..
            } => match task_type.as_str() {
                "token" => {
                    if page_action.is_none()
                        || data.is_none()
                        || page_data.is_none()
                        || user_agent.is_none()
                    {
                        return Err(CapmonsterError::InvalidTaskData(
                            "Turnstile challenge token mode requires pageAction, data, pageData and userAgent".to_string(),
                        ));
                    }
                }
                "cf_clearance" => {
                    if html_page_base64.is_none() || user_agent.is_none() || proxy.is_none() {
                        return Err(CapmonsterError::InvalidTaskData(
                            "Turnstile challenge cf_clearance mode requires htmlPageBase64, userAgent and proxy"
                                .to_string(),
                        ));
                    }
                }
                "wait_room" => {
                    if html_page_base64.is_none() || user_agent.is_none() || proxy.is_none() {
                        return Err(CapmonsterError::InvalidTaskData(
                            "Turnstile wait_room mode requires htmlPageBase64, userAgent and proxy"
                                .to_string(),
                        ));
                    }
                }
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    /// Create captcha task (internal).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "create_task_internal",
            target = "captcha.capmonster",
            skip_all,
            fields(task_id = tracing::field::Empty)
        )
    )]
    async fn create_task_internal(&self, task: CapmonsterTask) -> Result<TaskId> {
        Self::validate_task(&task)?;

        let request = CreateTaskRequest {
            client_key: self.api_key(),
            task: &task,
        };

        let response: CapmonsterResponse<CreateTaskData> =
            self.post(CREATE_TASK_PATH, &request).await?;
        let data = response.into_result().map_err(CapmonsterError::Api)?;
        let task_id = TaskId::from(data.task_id);

        #[cfg(feature = "tracing")]
        Span::current().record("task_id", task_id.as_ref());

        Ok(task_id)
    }

    /// Get task result (internal).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "get_task_result_internal",
            target = "captcha.capmonster",
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

        let response: CapmonsterResponse<GetTaskData<T>> =
            self.post(GET_TASK_RESULT_PATH, &request).await?;

        let data = response.into_result().map_err(CapmonsterError::Api)?;

        #[cfg(feature = "tracing")]
        if data.solution.is_some() {
            set_span_ok();
        }

        Ok(data.solution)
    }

    #[cfg(feature = "tracing")]
    fn record_error(e: &CapmonsterError) {
        if crate::errors::RetryableError::is_retryable(e) {
            tracing::warn!(error = %ErrorChain(e), "Capmonster transient error");
        } else {
            set_span_error(&ErrorChain(e));
            tracing::error!(error = %ErrorChain(e), "Capmonster operation failed");
        }
    }
}

impl Provider for CapmonsterProvider {
    type Solution = CapmonsterSolution;
    type Error = CapmonsterError;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "create_task",
            target = "captcha.capmonster",
            skip_all,
            fields(captcha.task_type)
        )
    )]
    async fn create_task(&self, task: CaptchaTask) -> Result<TaskCreationOutcome<Self::Solution>> {
        #[cfg(feature = "tracing")]
        Span::current().record("captcha.task_type", task.to_string());

        let internal_task: CapmonsterTask =
            task.try_into().map_err(CapmonsterError::UnsupportedTask)?;
        let result = self.create_task_internal(internal_task).await;

        #[cfg(feature = "tracing")]
        match &result {
            Ok(_) => set_span_ok(),
            Err(e) => Self::record_error(e),
        }

        result.map(TaskCreationOutcome::Pending)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "get_task_result",
            target = "captcha.capmonster",
            skip_all,
            fields(task_id = %task_id)
        )
    )]
    async fn get_task_result(&self, task_id: &TaskId) -> Result<Option<Self::Solution>> {
        let result = self.get_task_result_internal(task_id).await;

        #[cfg(feature = "tracing")]
        if let Err(ref e) = result {
            Self::record_error(e);
        }

        result
    }
}

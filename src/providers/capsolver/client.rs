use super::errors::{CapsolverError, Result};
use super::response::CapsolverResponse;
use super::types::{CapsolverTask, CreateTaskData, CreateTaskRequest, GetTaskData, GetTaskResultRequest};
use crate::types::TaskId;
use reqwest::Url;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

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
/// Handles HTTP requests and response parsing.
#[derive(Clone)]
pub struct CapsolverClient {
    http_client: ClientWithMiddleware,
    api_key: SecretString,
    url: Url,
}

impl Debug for CapsolverClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapsolverClient")
            .field("url", &self.url)
            .field("api_key", &"[REDACTED]")
            .finish()
    }
}

impl CapsolverClient {
    /// Create a new Capsolver client
    ///
    /// # Arguments
    /// * `url` - Base URL for the Capsolver API (e.g., `https://api.capsolver.com`)
    /// * `api_key` - Your Capsolver API key
    pub fn new(url: Url, api_key: impl Into<String>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(CapsolverError::BuildHttpClient)?;

        Ok(Self {
            http_client: ClientBuilder::new(client).build(),
            api_key: SecretString::from(api_key.into()),
            url,
        })
    }

    /// Create a new Capsolver client with a custom HTTP client
    ///
    /// Use this when you need to configure the HTTP client with custom middleware
    /// (e.g., tracing, retry, etc.)
    pub fn with_http_client(
        url: Url,
        api_key: impl Into<String>,
        http_client: ClientWithMiddleware,
    ) -> Self {
        Self {
            http_client,
            api_key: SecretString::from(api_key.into()),
            url,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::capsolver::errors::CapsolverErrorCode;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestSolution {
        #[serde(rename = "userAgent")]
        user_agent: String,
        #[serde(rename = "gRecaptchaResponse")]
        g_recaptcha_response: String,
    }

    #[tokio::test]
    async fn test_create_task_success() {
        let mock_server = MockServer::start().await;

        let response_body = json!({
            "errorId": 0,
            "errorCode": "",
            "errorDescription": "",
            "taskId": "37223a89-06ed-442c-a0b8-22067b79c5b4"
        });

        Mock::given(method("POST"))
            .and(path("/createTask"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let client =
            CapsolverClient::new(Url::parse(&mock_server.uri()).unwrap(), "test_api_key").unwrap();

        let task = CapsolverTask::turnstile("https://example.com", "test_key");

        let result = client.create_task(task).await;

        assert!(result.is_ok());
        let task_id = result.unwrap();
        assert_eq!(task_id.as_ref(), "37223a89-06ed-442c-a0b8-22067b79c5b4");
    }

    #[tokio::test]
    async fn test_create_task_api_error() {
        let mock_server = MockServer::start().await;

        let response_body = json!({
            "errorId": 1,
            "errorCode": "ERROR_ZERO_BALANCE",
            "description": "Insufficient balance"
        });

        Mock::given(method("POST"))
            .and(path("/createTask"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let client =
            CapsolverClient::new(Url::parse(&mock_server.uri()).unwrap(), "test_api_key").unwrap();

        let task = CapsolverTask::turnstile("https://example.com", "test_key");

        let result = client.create_task(task).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CapsolverError::Api(error) => {
                assert_eq!(error.error_id, 1);
                assert_eq!(error.error_code, CapsolverErrorCode::ZeroBalance);
            }
            _ => panic!("Expected Api error"),
        }
    }

    #[tokio::test]
    async fn test_get_task_result_ready() {
        let mock_server = MockServer::start().await;

        let response_body = json!({
            "errorId": 0,
            "taskId": "test-task-id",
            "solution": {
                "userAgent": "Mozilla/5.0...",
                "gRecaptchaResponse": "03AGdBq25SxXT-pmSeBXjzScW-EiocHwwpwqtk1QXlJnGnUJCZrgjwLLdt7cb0..."
            },
            "status": "ready"
        });

        Mock::given(method("POST"))
            .and(path("/getTaskResult"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let client =
            CapsolverClient::new(Url::parse(&mock_server.uri()).unwrap(), "test_api_key").unwrap();

        let task_id = TaskId::from("test-task-id");

        let result: Result<Option<TestSolution>> = client.get_task_result(&task_id).await;

        assert!(result.is_ok());
        let solution = result.unwrap();
        assert!(solution.is_some());
        let solution = solution.unwrap();
        assert_eq!(solution.user_agent, "Mozilla/5.0...");
        assert!(solution.g_recaptcha_response.starts_with("03AGdBq25SxXT"));
    }

    #[tokio::test]
    async fn test_get_task_result_processing() {
        let mock_server = MockServer::start().await;

        let response_body = json!({
            "errorId": 0,
            "taskId": "test-task-id",
            "status": "processing"
        });

        Mock::given(method("POST"))
            .and(path("/getTaskResult"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let client =
            CapsolverClient::new(Url::parse(&mock_server.uri()).unwrap(), "test_api_key").unwrap();

        let task_id = TaskId::from("test-task-id");

        let result: Result<Option<TestSolution>> = client.get_task_result(&task_id).await;

        assert!(result.is_ok());
        let solution = result.unwrap();
        assert!(solution.is_none());
    }

    #[tokio::test]
    async fn test_get_task_result_api_error() {
        let mock_server = MockServer::start().await;

        let response_body = json!({
            "errorId": 1,
            "errorCode": "ERROR_TASKID_INVALID",
            "description": "Task ID is invalid"
        });

        Mock::given(method("POST"))
            .and(path("/getTaskResult"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let client =
            CapsolverClient::new(Url::parse(&mock_server.uri()).unwrap(), "test_api_key").unwrap();

        let task_id = TaskId::from("invalid-task-id");

        let result: Result<Option<TestSolution>> = client.get_task_result(&task_id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CapsolverError::Api(error) => {
                assert_eq!(error.error_id, 1);
                assert_eq!(error.error_code, CapsolverErrorCode::TaskIdInvalid);
                assert_eq!(error.description, Some("Task ID is invalid".to_string()));
            }
            _ => panic!("Expected Api error"),
        }
    }

    #[test]
    fn test_error_code_retryability() {
        // Retryable errors
        assert!(CapsolverErrorCode::ServiceUnavailable.is_retryable());
        assert!(CapsolverErrorCode::RateLimit.is_retryable());
        assert!(CapsolverErrorCode::IpBanned.is_retryable());
        assert!(CapsolverErrorCode::KeyTempBlocked.is_retryable());

        // Non-retryable errors
        assert!(!CapsolverErrorCode::ZeroBalance.is_retryable());
        assert!(!CapsolverErrorCode::KeyDeniedAccess.is_retryable());
        assert!(!CapsolverErrorCode::InvalidTaskData.is_retryable());
        assert!(!CapsolverErrorCode::TaskIdInvalid.is_retryable());
        assert!(!CapsolverErrorCode::Unknown.is_retryable());
    }

    #[test]
    fn test_capsolver_response_deserialization_success() {
        let json = r#"{
            "errorId": 0,
            "errorCode": "",
            "errorDescription": "",
            "taskId": "37223a89-06ed-442c-a0b8-22067b79c5b4"
        }"#;

        let response: CapsolverResponse<CreateTaskData> = serde_json::from_str(json).unwrap();
        assert!(response.is_success());
        let data = response.into_result().unwrap();
        assert_eq!(data.task_id, "37223a89-06ed-442c-a0b8-22067b79c5b4");
    }

    #[test]
    fn test_capsolver_response_deserialization_error() {
        let json = r#"{
            "errorId": 1,
            "errorCode": "ERROR_ZERO_BALANCE",
            "description": "Error Description"
        }"#;

        let response: CapsolverResponse<CreateTaskData> = serde_json::from_str(json).unwrap();
        assert!(!response.is_success());
        let error = response.into_result().unwrap_err();
        assert_eq!(error.error_id, 1);
        assert_eq!(error.error_code, CapsolverErrorCode::ZeroBalance);
        assert_eq!(error.description, Some("Error Description".to_string()));
    }

    #[test]
    fn test_capsolver_response_get_task_ready() {
        let json = r#"{
            "errorId": 0,
            "taskId": "test-id",
            "solution": {
                "userAgent": "xxx",
                "gRecaptchaResponse": "03AGdBq25SxXT-pmSeBXjzScW-EiocHwwpwqtk1QXlJnGnUJCZrgjwLLdt7cb0..."
            },
            "status": "ready"
        }"#;

        let response: CapsolverResponse<GetTaskData<TestSolution>> =
            serde_json::from_str(json).unwrap();
        assert!(response.is_success());
        let data = response.into_result().unwrap();
        assert_eq!(data.status, "ready");
        assert!(data.solution.is_some());
        let solution = data.solution.unwrap();
        assert_eq!(solution.user_agent, "xxx");
    }

    #[test]
    fn test_capsolver_response_get_task_processing() {
        let json = r#"{
            "errorId": 0,
            "taskId": "test-id",
            "status": "processing"
        }"#;

        let response: CapsolverResponse<GetTaskData<TestSolution>> =
            serde_json::from_str(json).unwrap();
        assert!(response.is_success());
        let data = response.into_result().unwrap();
        assert_eq!(data.status, "processing");
        assert!(data.solution.is_none());
    }
}
//! Tests for the RuCaptcha provider and related functionality.

use super::errors::{RucaptchaError, RucaptchaErrorCode};
use super::provider::RucaptchaProvider;
use super::response::RucaptchaResponse;
use super::types::{CreateTaskData, GetTaskData, RucaptchaSolution};
use crate::provider::Provider;
use crate::tasks::Turnstile;
use crate::types::TaskId;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// =============================================================================
// Test Helpers
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestSolution {
    #[serde(rename = "gRecaptchaResponse")]
    g_recaptcha_response: String,
    #[serde(default)]
    token: Option<String>,
}

/// Create a mock provider connected to the given mock server
fn mock_provider(server: &MockServer) -> RucaptchaProvider {
    RucaptchaProvider::builder("test_api_key")
        .url(Url::parse(&server.uri()).unwrap())
        .build()
        .unwrap()
}

/// Mount a mock response for createTask endpoint
async fn mock_create_task(server: &MockServer, response: Value) {
    Mock::given(method("POST"))
        .and(path("/createTask"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response))
        .mount(server)
        .await;
}

/// Mount a mock response for getTaskResult endpoint
async fn mock_get_task_result(server: &MockServer, response: Value) {
    Mock::given(method("POST"))
        .and(path("/getTaskResult"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response))
        .mount(server)
        .await;
}

/// Create a success response for createTask
fn success_create_task_response(task_id: &str) -> Value {
    json!({
        "errorId": 0,
        "errorCode": "",
        "errorDescription": "",
        "taskId": task_id
    })
}

/// Create an error response
fn error_response(error_code: &str, description: &str) -> Value {
    json!({
        "errorId": 1,
        "errorCode": error_code,
        "errorDescription": description
    })
}

/// Create a ready solution response
fn ready_solution_response(task_id: &str, solution: Value) -> Value {
    json!({
        "errorId": 0,
        "taskId": task_id,
        "solution": solution,
        "status": "ready"
    })
}

/// Create a processing response
fn processing_response(task_id: &str) -> Value {
    json!({
        "errorId": 0,
        "taskId": task_id,
        "status": "processing"
    })
}

// =============================================================================
// Provider Tests
// =============================================================================

#[tokio::test]
async fn test_create_task_success() {
    let server = MockServer::start().await;
    mock_create_task(
        &server,
        success_create_task_response("37223a89-06ed-442c-a0b8-22067b79c5b4"),
    )
    .await;

    let provider = mock_provider(&server);
    let task = Turnstile::new("https://example.com", "test_key");

    let task_id = provider.create_task(task.into()).await.unwrap();
    assert_eq!(task_id.as_ref(), "37223a89-06ed-442c-a0b8-22067b79c5b4");
}

#[tokio::test]
async fn test_create_task_api_error() {
    let server = MockServer::start().await;
    mock_create_task(
        &server,
        error_response("ERROR_ZERO_BALANCE", "Insufficient balance"),
    )
    .await;

    let provider = mock_provider(&server);
    let task = Turnstile::new("https://example.com", "test_key");

    let err = provider.create_task(task.into()).await.unwrap_err();
    match err {
        RucaptchaError::Api(error) => {
            assert_eq!(error.error_id, 1);
            assert_eq!(error.error_code, RucaptchaErrorCode::ZeroBalance);
        }
        _ => panic!("Expected Api error"),
    }
}

#[tokio::test]
async fn test_get_task_result_ready() {
    let server = MockServer::start().await;
    let solution = json!({
        "gRecaptchaResponse": "03AGdBq25SxXT-pmSeBXjzScW-EiocHwwpwqtk1QXlJnGnUJCZrgjwLLdt7cb0...",
        "token": "03AGdBq25SxXT-pmSeBXjzScW-EiocHwwpwqtk1QXlJnGnUJCZrgjwLLdt7cb0..."
    });
    mock_get_task_result(&server, ready_solution_response("test-task-id", solution)).await;

    let provider = mock_provider(&server);
    let task_id = TaskId::from("test-task-id");

    let solution: Option<RucaptchaSolution> = provider.get_task_result(&task_id).await.unwrap();
    let solution = solution.unwrap();
    let recaptcha = solution.into_recaptcha();
    assert!(recaptcha.token().starts_with("03AGdBq25SxXT"));
}

#[tokio::test]
async fn test_get_task_result_processing() {
    let server = MockServer::start().await;
    mock_get_task_result(&server, processing_response("test-task-id")).await;

    let provider = mock_provider(&server);
    let task_id = TaskId::from("test-task-id");

    let solution: Option<RucaptchaSolution> = provider.get_task_result(&task_id).await.unwrap();
    assert!(solution.is_none());
}

#[tokio::test]
async fn test_get_task_result_api_error() {
    let server = MockServer::start().await;
    mock_get_task_result(
        &server,
        error_response("ERROR_NO_SUCH_CAPCHA_ID", "Task ID is invalid"),
    )
    .await;

    let provider = mock_provider(&server);
    let task_id = TaskId::from("invalid-task-id");

    let err: RucaptchaError = provider.get_task_result(&task_id).await.unwrap_err();
    match err {
        RucaptchaError::Api(error) => {
            assert_eq!(error.error_id, 1);
            assert_eq!(error.error_code, RucaptchaErrorCode::NoSuchCaptchaId);
            assert_eq!(
                error.error_description,
                Some("Task ID is invalid".to_string())
            );
        }
        _ => panic!("Expected Api error"),
    }
}

// =============================================================================
// Builder Tests
// =============================================================================

#[test]
fn test_builder_default_url() {
    let provider = RucaptchaProvider::new("test-key").unwrap();
    assert_eq!(provider.url().as_str(), "https://api.rucaptcha.com/");
}

#[test]
fn test_builder_custom_url() {
    let custom_url = Url::parse("https://custom.example.com").unwrap();
    let provider = RucaptchaProvider::builder("test-key")
        .url(custom_url.clone())
        .build()
        .unwrap();
    assert_eq!(*provider.url(), custom_url);
}

// =============================================================================
// Error Code Tests
// =============================================================================

#[test]
fn test_error_code_retryability() {
    // Task-level retryable errors (same task can be retried)
    assert!(RucaptchaErrorCode::NoSlotAvailable.is_retryable());

    // Non-retryable at task level
    assert!(!RucaptchaErrorCode::ZeroBalance.is_retryable());
    assert!(!RucaptchaErrorCode::CaptchaUnsolvable.is_retryable());
    assert!(!RucaptchaErrorCode::KeyDoesNotExist.is_retryable());
    assert!(!RucaptchaErrorCode::BadParameters.is_retryable());
    assert!(!RucaptchaErrorCode::NoSuchCaptchaId.is_retryable());
    assert!(!RucaptchaErrorCode::TaskNotSupported.is_retryable());
    assert!(!RucaptchaErrorCode::Unknown.is_retryable());

    // Operation-level retryable (fresh task might succeed)
    assert!(RucaptchaErrorCode::NoSlotAvailable.should_retry_operation());
    assert!(RucaptchaErrorCode::CaptchaUnsolvable.should_retry_operation());

    // Not retryable at operation level (need to fix account/configuration)
    assert!(!RucaptchaErrorCode::ZeroBalance.should_retry_operation());
    assert!(!RucaptchaErrorCode::KeyDoesNotExist.should_retry_operation());
}

// =============================================================================
// Response Deserialization Tests
// =============================================================================

#[test]
fn test_rucaptcha_response_deserialization_success() {
    let json = r#"{
        "errorId": 0,
        "errorCode": "",
        "errorDescription": "",
        "taskId": "37223a89-06ed-442c-a0b8-22067b79c5b4"
    }"#;

    let response: RucaptchaResponse<CreateTaskData> = serde_json::from_str(json).unwrap();
    let data = response.into_result().expect("expected success response");
    assert_eq!(data.task_id, "37223a89-06ed-442c-a0b8-22067b79c5b4");
}

#[test]
fn test_rucaptcha_response_deserialization_error() {
    let json = r#"{
        "errorId": 1,
        "errorCode": "ERROR_ZERO_BALANCE",
        "errorDescription": "Error Description"
    }"#;

    let response: RucaptchaResponse<CreateTaskData> = serde_json::from_str(json).unwrap();
    let error = response.into_result().expect_err("expected error response");
    assert_eq!(error.error_id, 1);
    assert_eq!(error.error_code, RucaptchaErrorCode::ZeroBalance);
    assert_eq!(
        error.error_description,
        Some("Error Description".to_string())
    );
}

#[test]
fn test_rucaptcha_response_get_task_ready() {
    let json = r#"{
        "errorId": 0,
        "taskId": "test-id",
        "solution": {
            "gRecaptchaResponse": "03AGdBq25SxXT-pmSeBXjzScW-EiocHwwpwqtk1QXlJnGnUJCZrgjwLLdt7cb0...",
            "token": "03AGdBq25SxXT-pmSeBXjzScW-EiocHwwpwqtk1QXlJnGnUJCZrgjwLLdt7cb0..."
        },
        "status": "ready"
    }"#;

    let response: RucaptchaResponse<GetTaskData<TestSolution>> =
        serde_json::from_str(json).unwrap();
    let data = response.into_result().expect("expected success response");
    assert_eq!(data.status, "ready");
    let solution = data.solution.expect("expected solution");
    assert!(solution.g_recaptcha_response.starts_with("03AGdBq25SxXT"));
}

#[test]
fn test_rucaptcha_response_get_task_processing() {
    let json = r#"{
        "errorId": 0,
        "taskId": "test-id",
        "status": "processing"
    }"#;

    let response: RucaptchaResponse<GetTaskData<TestSolution>> =
        serde_json::from_str(json).unwrap();
    let data = response.into_result().expect("expected success response");
    assert_eq!(data.status, "processing");
    assert!(data.solution.is_none());
}

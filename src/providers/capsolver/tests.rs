//! Tests for the Capsolver client and related functionality.

use super::client::CapsolverClient;
use super::errors::{CapsolverError, CapsolverErrorCode};
use super::response::CapsolverResponse;
use super::types::{CapsolverTask, CreateTaskData, GetTaskData};
use crate::types::TaskId;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// =============================================================================
// Test Helpers
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestSolution {
    #[serde(rename = "userAgent")]
    user_agent: String,
    #[serde(rename = "gRecaptchaResponse")]
    g_recaptcha_response: String,
}

/// Create a mock client connected to the given mock server
fn mock_client(server: &MockServer) -> CapsolverClient {
    CapsolverClient::builder("test_api_key")
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
        "description": description
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
// Client Tests
// =============================================================================

#[tokio::test]
async fn test_create_task_success() {
    let server = MockServer::start().await;
    mock_create_task(
        &server,
        success_create_task_response("37223a89-06ed-442c-a0b8-22067b79c5b4"),
    )
    .await;

    let client = mock_client(&server);
    let task = CapsolverTask::turnstile("https://example.com", "test_key");

    let task_id = client.create_task(task).await.unwrap();
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

    let client = mock_client(&server);
    let task = CapsolverTask::turnstile("https://example.com", "test_key");

    let err = client.create_task(task).await.unwrap_err();
    match err {
        CapsolverError::Api(error) => {
            assert_eq!(error.error_id, 1);
            assert_eq!(error.error_code, CapsolverErrorCode::ZeroBalance);
        }
        _ => panic!("Expected Api error"),
    }
}

#[tokio::test]
async fn test_get_task_result_ready() {
    let server = MockServer::start().await;
    let solution = json!({
        "userAgent": "Mozilla/5.0...",
        "gRecaptchaResponse": "03AGdBq25SxXT-pmSeBXjzScW-EiocHwwpwqtk1QXlJnGnUJCZrgjwLLdt7cb0..."
    });
    mock_get_task_result(&server, ready_solution_response("test-task-id", solution)).await;

    let client = mock_client(&server);
    let task_id = TaskId::from("test-task-id");

    let solution: Option<TestSolution> = client.get_task_result(&task_id).await.unwrap();
    let solution = solution.unwrap();
    assert_eq!(solution.user_agent, "Mozilla/5.0...");
    assert!(solution.g_recaptcha_response.starts_with("03AGdBq25SxXT"));
}

#[tokio::test]
async fn test_get_task_result_processing() {
    let server = MockServer::start().await;
    mock_get_task_result(&server, processing_response("test-task-id")).await;

    let client = mock_client(&server);
    let task_id = TaskId::from("test-task-id");

    let solution: Option<TestSolution> = client.get_task_result(&task_id).await.unwrap();
    assert!(solution.is_none());
}

#[tokio::test]
async fn test_get_task_result_api_error() {
    let server = MockServer::start().await;
    mock_get_task_result(
        &server,
        error_response("ERROR_TASKID_INVALID", "Task ID is invalid"),
    )
    .await;

    let client = mock_client(&server);
    let task_id = TaskId::from("invalid-task-id");

    let err: CapsolverError = client
        .get_task_result::<TestSolution>(&task_id)
        .await
        .unwrap_err();
    match err {
        CapsolverError::Api(error) => {
            assert_eq!(error.error_id, 1);
            assert_eq!(error.error_code, CapsolverErrorCode::TaskIdInvalid);
            assert_eq!(error.description, Some("Task ID is invalid".to_string()));
        }
        _ => panic!("Expected Api error"),
    }
}

// =============================================================================
// Builder Tests
// =============================================================================

#[test]
fn test_builder_default_url() {
    let client = CapsolverClient::new("test-key").unwrap();
    assert_eq!(client.url.as_str(), "https://api.capsolver.com/");
}

#[test]
fn test_builder_custom_url() {
    let custom_url = Url::parse("https://custom.example.com").unwrap();
    let client = CapsolverClient::builder("test-key")
        .url(custom_url.clone())
        .build()
        .unwrap();
    assert_eq!(client.url, custom_url);
}

// =============================================================================
// Error Code Tests
// =============================================================================

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

// =============================================================================
// Response Deserialization Tests
// =============================================================================

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

    let response: CapsolverResponse<GetTaskData<TestSolution>> = serde_json::from_str(json).unwrap();
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

    let response: CapsolverResponse<GetTaskData<TestSolution>> = serde_json::from_str(json).unwrap();
    assert!(response.is_success());
    let data = response.into_result().unwrap();
    assert_eq!(data.status, "processing");
    assert!(data.solution.is_none());
}
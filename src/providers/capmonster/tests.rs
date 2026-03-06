//! Tests for the CapMonster provider.

use super::errors::{CapmonsterError, CapmonsterErrorCode};
use super::provider::CapmonsterProvider;
use super::response::CapmonsterResponse;
use super::types::{CapmonsterSolution, CreateTaskData, GetTaskData};
use crate::providers::traits::Provider;
use crate::tasks::{Turnstile, TurnstileChallenge, TurnstileChallengeMode};
use crate::utils::types::TaskId;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestSolution {
    #[serde(rename = "gRecaptchaResponse")]
    g_recaptcha_response: String,
}

fn mock_provider(server: &MockServer) -> CapmonsterProvider {
    CapmonsterProvider::builder("test_api_key")
        .url(Url::parse(&server.uri()).unwrap())
        .build()
        .unwrap()
}

async fn mock_create_task(server: &MockServer, response: Value) {
    Mock::given(method("POST"))
        .and(path("/createTask"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response))
        .mount(server)
        .await;
}

async fn mock_get_task_result(server: &MockServer, response: Value) {
    Mock::given(method("POST"))
        .and(path("/getTaskResult"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response))
        .mount(server)
        .await;
}

fn success_create_task_response(task_id: &str) -> Value {
    json!({
        "errorId": 0,
        "taskId": task_id,
    })
}

fn error_response(error_code: &str, description: &str) -> Value {
    json!({
        "errorId": 1,
        "errorCode": error_code,
        "errorDescription": description,
    })
}

fn ready_solution_response(task_id: &str, solution: Value) -> Value {
    json!({
        "errorId": 0,
        "taskId": task_id,
        "status": "ready",
        "solution": solution,
    })
}

fn processing_response(task_id: &str) -> Value {
    json!({
        "errorId": 0,
        "taskId": task_id,
        "status": "processing",
    })
}

#[tokio::test]
async fn test_create_task_success() {
    let server = MockServer::start().await;
    mock_create_task(
        &server,
        success_create_task_response("37223a89-06ed-442c-a0b8-22067b79c5b4"),
    )
    .await;

    let provider = mock_provider(&server);
    let task = Turnstile::new("https://example.com", "site-key");

    let outcome = provider.create_task(task.into()).await.unwrap();
    assert_eq!(
        outcome.task_id().as_ref(),
        "37223a89-06ed-442c-a0b8-22067b79c5b4"
    );
}

#[tokio::test]
async fn test_create_task_api_error() {
    let server = MockServer::start().await;
    mock_create_task(
        &server,
        error_response("ERROR_KEY_DOES_NOT_EXIST", "Invalid API key"),
    )
    .await;

    let provider = mock_provider(&server);
    let task = Turnstile::new("https://example.com", "site-key");
    let err = provider.create_task(task.into()).await.unwrap_err();

    match err {
        CapmonsterError::Api(error) => {
            assert_eq!(error.error_code, CapmonsterErrorCode::KeyDoesNotExist);
        }
        _ => panic!("Expected Api error"),
    }
}

#[tokio::test]
async fn test_create_task_invalid_token_challenge_data() {
    let provider = CapmonsterProvider::new("test_api_key").unwrap();
    let task = TurnstileChallenge {
        website_url: "https://example.com".to_string(),
        website_key: "site-key".to_string(),
        mode: TurnstileChallengeMode::Token,
        page_action: Some("managed".to_string()),
        data: Some("cdata".to_string()),
        page_data: None,
        api_js_url: None,
        html_page_base64: None,
        user_agent: "Mozilla/5.0".to_string(),
        proxy: None,
    };

    let err = provider.create_task(task.into()).await.unwrap_err();
    assert!(matches!(err, CapmonsterError::InvalidTaskData(_)));
}

#[tokio::test]
async fn test_get_task_result_ready() {
    let server = MockServer::start().await;
    let solution = json!({
        "gRecaptchaResponse": "recaptcha-token"
    });

    mock_get_task_result(&server, ready_solution_response("test-task-id", solution)).await;

    let provider = mock_provider(&server);
    let task_id = TaskId::from("test-task-id");

    let solution: Option<CapmonsterSolution> = provider.get_task_result(&task_id).await.unwrap();
    let solution = solution.unwrap();
    let recaptcha = solution.into_recaptcha();
    assert_eq!(recaptcha.token(), "recaptcha-token");
}

#[tokio::test]
async fn test_get_task_result_processing() {
    let server = MockServer::start().await;
    mock_get_task_result(&server, processing_response("test-task-id")).await;

    let provider = mock_provider(&server);
    let task_id = TaskId::from("test-task-id");

    let solution: Option<CapmonsterSolution> = provider.get_task_result(&task_id).await.unwrap();
    assert!(solution.is_none());
}

#[test]
fn test_builder_default_url() {
    let provider = CapmonsterProvider::new("test-key").unwrap();
    assert_eq!(provider.url().as_str(), "https://api.capmonster.cloud/");
}

#[test]
fn test_builder_custom_url() {
    let custom_url = Url::parse("https://custom.example.com").unwrap();
    let provider = CapmonsterProvider::builder("test-key")
        .url(custom_url.clone())
        .build()
        .unwrap();
    assert_eq!(*provider.url(), custom_url);
}

#[test]
fn test_response_deserialization_success() {
    let json = r#"{
        "errorId": 0,
        "taskId": "37223a89-06ed-442c-a0b8-22067b79c5b4"
    }"#;

    let response: CapmonsterResponse<CreateTaskData> = serde_json::from_str(json).unwrap();
    let data = response.into_result().expect("expected success response");
    assert_eq!(data.task_id, "37223a89-06ed-442c-a0b8-22067b79c5b4");
}

#[test]
fn test_response_deserialization_error() {
    let json = r#"{
        "errorId": 1,
        "errorCode": "ERROR_KEY_DOES_NOT_EXIST",
        "errorDescription": "Invalid API key"
    }"#;

    let response: CapmonsterResponse<CreateTaskData> = serde_json::from_str(json).unwrap();
    let error = response.into_result().expect_err("expected error response");
    assert_eq!(error.error_code, CapmonsterErrorCode::KeyDoesNotExist);
}

#[test]
fn test_response_get_task_ready() {
    let json = r#"{
        "errorId": 0,
        "taskId": "test-id",
        "status": "ready",
        "solution": {"gRecaptchaResponse": "token"}
    }"#;

    let response: CapmonsterResponse<GetTaskData<TestSolution>> =
        serde_json::from_str(json).unwrap();
    let data = response.into_result().expect("expected success response");
    assert_eq!(data.status, "ready");
    assert_eq!(
        data.solution
            .expect("expected solution")
            .g_recaptcha_response,
        "token"
    );
}

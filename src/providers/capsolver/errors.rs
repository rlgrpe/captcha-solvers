use crate::errors::RetryableError;
use crate::types::TaskId;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CapsolverError {
    #[error("Failed to build HTTP client: {0}")]
    BuildHttpClient(#[source] reqwest::Error),

    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest_middleware::Error),

    #[error("Failed to parse response: {0}")]
    ParseResponse(#[source] reqwest::Error),

    #[error("Capsolver API error: {0}")]
    Api(#[source] CapsolverApiError),

    #[error(
        "Timeout waiting for captcha solution after {:.1}s; Task id: {task_id}",
        timeout.as_secs_f64()
    )]
    SolutionTimeout { timeout: Duration, task_id: TaskId },
}

pub type Result<T> = std::result::Result<T, CapsolverError>;

impl RetryableError for CapsolverError {
    fn is_retryable(&self) -> bool {
        match self {
            // Retryable HTTP/network errors
            CapsolverError::HttpRequest(_) => true,
            // Timeouts are considered retryable
            CapsolverError::SolutionTimeout { .. } => true,
            // API errors are retryable based on error code
            CapsolverError::Api(error) => error.error_code.is_retryable(),
            // Non-retryable errors
            CapsolverError::BuildHttpClient(_) | CapsolverError::ParseResponse(_) => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CapsolverErrorCode {
    // === Transient / Server Errors (Retryable) ===
    #[serde(rename = "ERROR_SERVICE_UNAVALIABLE")]
    ServiceUnavailable,
    #[serde(rename = "ERROR_RATE_LIMIT")]
    RateLimit,
    #[serde(rename = "ERROR_IP_BANNED")]
    IpBanned,
    #[serde(rename = "ERROR_KEY_TEMP_BLOCKED")]
    KeyTempBlocked,

    // === Fatal / Client Errors (Non-retryable) ===
    #[serde(rename = "ERROR_ZERO_BALANCE")]
    ZeroBalance,
    #[serde(rename = "ERROR_KEY_DENIED_ACCESS")]
    KeyDeniedAccess,
    #[serde(rename = "ERROR_INVALID_TASK_DATA")]
    InvalidTaskData,
    #[serde(rename = "ERROR_BAD_REQUEST")]
    BadRequest,
    #[serde(rename = "ERROR_TASKID_INVALID")]
    TaskIdInvalid,
    #[serde(rename = "ERROR_TASK_NOT_FOUND")]
    TaskNotFound,
    #[serde(rename = "ERROR_TASK_NOT_SUPPORTED")]
    TaskNotSupported,
    #[serde(rename = "ERROR_UNKNOWN_QUESTION")]
    UnknownQuestion,
    #[serde(rename = "ERROR_PROXY_BANNED")]
    ProxyBanned,
    #[serde(rename = "ERROR_INVALID_IMAGE")]
    InvalidImage,
    #[serde(rename = "ERROR_PARSE_IMAGE_FAIL")]
    ParseImageFail,

    // === Logic Errors (Task result) ===
    #[serde(rename = "ERROR_TASK_TIMEOUT")]
    TaskTimeout,
    #[serde(rename = "ERROR_CAPTCHA_UNSOLVABLE")]
    CaptchaUnsolvable,
    #[serde(rename = "ERROR_SETTLEMENT_FAILED")]
    SettlementFailed,

    // Fallback for new API errors not in documentation
    #[serde(other)]
    Unknown,
}

impl CapsolverErrorCode {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ServiceUnavailable
                | Self::RateLimit
                | Self::IpBanned
                | Self::KeyTempBlocked
                | Self::TaskNotFound
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapsolverApiError {
    pub error_id: u32,
    pub error_code: CapsolverErrorCode,
    #[serde(default)]
    pub description: Option<String>,
}

impl fmt::Display for CapsolverApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Capsolver Error [{}]: {:?} - {}",
            self.error_id,
            self.error_code,
            self.description
                .clone()
                .unwrap_or_else(|| "No description".to_string())
        )
    }
}

impl std::error::Error for CapsolverApiError {}
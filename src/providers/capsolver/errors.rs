use crate::errors::RetryableError;
use crate::types::TaskId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

/// Known Capsolver error codes.
///
/// When API returns an unknown error code, it will be stored in `Other(String)`
/// variant with the original error code string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapsolverErrorCode {
    // === Transient / Server Errors (Retryable) ===
    ServiceUnavailable,
    RateLimit,
    IpBanned,
    KeyTempBlocked,

    // === Fatal / Client Errors (Non-retryable) ===
    ZeroBalance,
    KeyDeniedAccess,
    InvalidTaskData,
    BadRequest,
    TaskIdInvalid,
    TaskNotFound,
    TaskNotSupported,
    UnknownQuestion,
    ProxyBanned,
    InvalidImage,
    ParseImageFail,

    // === Logic Errors (Task result) ===
    TaskTimeout,
    CaptchaUnsolvable,
    SettlementFailed,

    /// Fallback for new API errors not in documentation.
    /// Contains the original error code string from the API.
    Other(String),
}

impl CapsolverErrorCode {
    /// Returns the API error code string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::ServiceUnavailable => "ERROR_SERVICE_UNAVALIABLE",
            Self::RateLimit => "ERROR_RATE_LIMIT",
            Self::IpBanned => "ERROR_IP_BANNED",
            Self::KeyTempBlocked => "ERROR_KEY_TEMP_BLOCKED",
            Self::ZeroBalance => "ERROR_ZERO_BALANCE",
            Self::KeyDeniedAccess => "ERROR_KEY_DENIED_ACCESS",
            Self::InvalidTaskData => "ERROR_INVALID_TASK_DATA",
            Self::BadRequest => "ERROR_BAD_REQUEST",
            Self::TaskIdInvalid => "ERROR_TASKID_INVALID",
            Self::TaskNotFound => "ERROR_TASK_NOT_FOUND",
            Self::TaskNotSupported => "ERROR_TASK_NOT_SUPPORTED",
            Self::UnknownQuestion => "ERROR_UNKNOWN_QUESTION",
            Self::ProxyBanned => "ERROR_PROXY_BANNED",
            Self::InvalidImage => "ERROR_INVALID_IMAGE",
            Self::ParseImageFail => "ERROR_PARSE_IMAGE_FAIL",
            Self::TaskTimeout => "ERROR_TASK_TIMEOUT",
            Self::CaptchaUnsolvable => "ERROR_CAPTCHA_UNSOLVABLE",
            Self::SettlementFailed => "ERROR_SETTLEMENT_FAILED",
            Self::Other(code) => code.as_str(),
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "ERROR_SERVICE_UNAVALIABLE" => Self::ServiceUnavailable,
            "ERROR_RATE_LIMIT" => Self::RateLimit,
            "ERROR_IP_BANNED" => Self::IpBanned,
            "ERROR_KEY_TEMP_BLOCKED" => Self::KeyTempBlocked,
            "ERROR_ZERO_BALANCE" => Self::ZeroBalance,
            "ERROR_KEY_DENIED_ACCESS" => Self::KeyDeniedAccess,
            "ERROR_INVALID_TASK_DATA" => Self::InvalidTaskData,
            "ERROR_BAD_REQUEST" => Self::BadRequest,
            "ERROR_TASKID_INVALID" => Self::TaskIdInvalid,
            "ERROR_TASK_NOT_FOUND" => Self::TaskNotFound,
            "ERROR_TASK_NOT_SUPPORTED" => Self::TaskNotSupported,
            "ERROR_UNKNOWN_QUESTION" => Self::UnknownQuestion,
            "ERROR_PROXY_BANNED" => Self::ProxyBanned,
            "ERROR_INVALID_IMAGE" => Self::InvalidImage,
            "ERROR_PARSE_IMAGE_FAIL" => Self::ParseImageFail,
            "ERROR_TASK_TIMEOUT" => Self::TaskTimeout,
            "ERROR_CAPTCHA_UNSOLVABLE" => Self::CaptchaUnsolvable,
            "ERROR_SETTLEMENT_FAILED" => Self::SettlementFailed,
            other => Self::Other(other.to_string()),
        }
    }
}

impl fmt::Display for CapsolverErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for CapsolverErrorCode {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for CapsolverErrorCode {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::from_str(&s))
    }
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
    pub error_description: Option<String>,
}

impl fmt::Display for CapsolverApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Capsolver Error [{}]: {} - {}",
            self.error_id,
            self.error_code,
            self.error_description
                .as_deref()
                .unwrap_or("No description")
        )
    }
}

impl std::error::Error for CapsolverApiError {}
use crate::errors::{RetryableError, UnsupportedTaskError};
use crate::utils::types::TaskId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CapmonsterError {
    #[error("Failed to build HTTP client: {0}")]
    BuildHttpClient(#[source] reqwest::Error),

    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest_middleware::Error),

    #[error("Failed to parse response: {0}")]
    ParseResponse(#[source] reqwest::Error),

    #[error("CapMonster API error: {0}")]
    Api(#[source] CapmonsterApiError),

    #[error("{0}")]
    UnsupportedTask(#[source] UnsupportedTaskError),

    #[error("Invalid task data: {0}")]
    InvalidTaskData(String),

    #[error(
        "Timeout waiting for captcha solution after {:.1}s; Task id: {task_id}",
        timeout.as_secs_f64()
    )]
    SolutionTimeout { timeout: Duration, task_id: TaskId },
}

pub type Result<T> = std::result::Result<T, CapmonsterError>;

impl RetryableError for CapmonsterError {
    fn is_retryable(&self) -> bool {
        match self {
            CapmonsterError::HttpRequest(_) => true,
            CapmonsterError::Api(error) => error.error_code.is_retryable(),
            CapmonsterError::BuildHttpClient(_)
            | CapmonsterError::ParseResponse(_)
            | CapmonsterError::UnsupportedTask(_)
            | CapmonsterError::InvalidTaskData(_)
            | CapmonsterError::SolutionTimeout { .. } => false,
        }
    }

    fn should_retry_operation(&self) -> bool {
        match self {
            CapmonsterError::HttpRequest(_) => true,
            CapmonsterError::Api(error) => error.error_code.should_retry_operation(),
            CapmonsterError::SolutionTimeout { .. } => true,
            CapmonsterError::BuildHttpClient(_)
            | CapmonsterError::ParseResponse(_)
            | CapmonsterError::UnsupportedTask(_)
            | CapmonsterError::InvalidTaskData(_) => false,
        }
    }
}

/// Known CapMonster API error codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapmonsterErrorCode {
    // Retryable/transient
    ServiceUnavailable,
    RateLimit,
    IpBanned,
    KeyTempBlocked,
    NoSlotAvailable,

    // Account/configuration
    KeyDoesNotExist,
    ZeroBalance,
    KeyDeniedAccess,
    BadParameters,
    BadProxy,

    // Task-level
    InvalidTaskData,
    BadRequest,
    TaskIdInvalid,
    TaskNotFound,
    TaskNotSupported,
    TaskTimeout,
    CaptchaUnsolvable,

    // Unknown
    Other(String),
}

impl CapmonsterErrorCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ServiceUnavailable => "ERROR_SERVICE_UNAVAILABLE",
            Self::RateLimit => "ERROR_RATE_LIMIT",
            Self::IpBanned => "ERROR_IP_BANNED",
            Self::KeyTempBlocked => "ERROR_KEY_TEMP_BLOCKED",
            Self::NoSlotAvailable => "ERROR_NO_SLOT_AVAILABLE",
            Self::KeyDoesNotExist => "ERROR_KEY_DOES_NOT_EXIST",
            Self::ZeroBalance => "ERROR_ZERO_BALANCE",
            Self::KeyDeniedAccess => "ERROR_KEY_DENIED_ACCESS",
            Self::BadParameters => "ERROR_BAD_PARAMETERS",
            Self::BadProxy => "ERROR_BAD_PROXY",
            Self::InvalidTaskData => "ERROR_INVALID_TASK_DATA",
            Self::BadRequest => "ERROR_BAD_REQUEST",
            Self::TaskIdInvalid => "ERROR_TASKID_INVALID",
            Self::TaskNotFound => "ERROR_TASK_NOT_FOUND",
            Self::TaskNotSupported => "ERROR_TASK_NOT_SUPPORTED",
            Self::TaskTimeout => "ERROR_TASK_TIMEOUT",
            Self::CaptchaUnsolvable => "ERROR_CAPTCHA_UNSOLVABLE",
            Self::Other(code) => code.as_str(),
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            // CapMonster/Capsolver variants with typo compatibility
            "ERROR_SERVICE_UNAVAILABLE" | "ERROR_SERVICE_UNAVALIABLE" => Self::ServiceUnavailable,
            "ERROR_RATE_LIMIT" => Self::RateLimit,
            "ERROR_IP_BANNED" => Self::IpBanned,
            "ERROR_KEY_TEMP_BLOCKED" => Self::KeyTempBlocked,
            "ERROR_NO_SLOT_AVAILABLE" => Self::NoSlotAvailable,
            "ERROR_KEY_DOES_NOT_EXIST" => Self::KeyDoesNotExist,
            "ERROR_ZERO_BALANCE" => Self::ZeroBalance,
            "ERROR_KEY_DENIED_ACCESS" => Self::KeyDeniedAccess,
            "ERROR_BAD_PARAMETERS" => Self::BadParameters,
            "ERROR_BAD_PROXY" => Self::BadProxy,
            "ERROR_INVALID_TASK_DATA" => Self::InvalidTaskData,
            "ERROR_BAD_REQUEST" => Self::BadRequest,
            "ERROR_TASKID_INVALID" => Self::TaskIdInvalid,
            "ERROR_TASK_NOT_FOUND" => Self::TaskNotFound,
            "ERROR_TASK_NOT_SUPPORTED" => Self::TaskNotSupported,
            "ERROR_TASK_TIMEOUT" => Self::TaskTimeout,
            "ERROR_CAPTCHA_UNSOLVABLE" => Self::CaptchaUnsolvable,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ServiceUnavailable
                | Self::RateLimit
                | Self::IpBanned
                | Self::KeyTempBlocked
                | Self::NoSlotAvailable
        )
    }

    pub fn should_retry_operation(&self) -> bool {
        match self {
            Self::ServiceUnavailable
            | Self::RateLimit
            | Self::IpBanned
            | Self::KeyTempBlocked
            | Self::NoSlotAvailable
            | Self::TaskNotFound
            | Self::TaskTimeout
            | Self::CaptchaUnsolvable => true,
            Self::KeyDoesNotExist
            | Self::ZeroBalance
            | Self::KeyDeniedAccess
            | Self::BadParameters
            | Self::BadProxy
            | Self::InvalidTaskData
            | Self::BadRequest
            | Self::TaskIdInvalid
            | Self::TaskNotSupported
            | Self::Other(_) => false,
        }
    }
}

impl fmt::Display for CapmonsterErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for CapmonsterErrorCode {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for CapmonsterErrorCode {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::from_str(&s))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapmonsterApiError {
    pub error_id: u32,
    pub error_code: CapmonsterErrorCode,
    #[serde(default)]
    pub error_description: Option<String>,
}

impl fmt::Display for CapmonsterApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CapMonster Error [{}]: {} - {}",
            self.error_id,
            self.error_code,
            self.error_description
                .as_deref()
                .unwrap_or("No description")
        )
    }
}

impl std::error::Error for CapmonsterApiError {}

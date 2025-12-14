//! Error types for the RuCaptcha provider.

use crate::errors::{RetryableError, UnsupportedTaskError};
use crate::utils::types::TaskId;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use thiserror::Error;

/// RuCaptcha error type
#[derive(Debug, Error)]
pub enum RucaptchaError {
    #[error("Failed to build HTTP client: {0}")]
    BuildHttpClient(#[source] reqwest::Error),

    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest_middleware::Error),

    #[error("Failed to parse response: {0}")]
    ParseResponse(#[source] reqwest::Error),

    #[error("RuCaptcha API error: {0}")]
    Api(#[source] RucaptchaApiError),

    #[error("{0}")]
    UnsupportedTask(#[source] UnsupportedTaskError),

    #[error(
        "Timeout waiting for captcha solution after {:.1}s; Task id: {task_id}",
        timeout.as_secs_f64()
    )]
    SolutionTimeout { timeout: Duration, task_id: TaskId },
}

pub type Result<T> = std::result::Result<T, RucaptchaError>;

impl RetryableError for RucaptchaError {
    fn is_retryable(&self) -> bool {
        match self {
            // Retryable HTTP/network errors
            RucaptchaError::HttpRequest(_) => true,
            // Timeouts are NOT retryable at task level (task already expired)
            RucaptchaError::SolutionTimeout { .. } => false,
            // API errors are retryable based on error code
            RucaptchaError::Api(error) => error.error_code.is_retryable(),
            // Non-retryable errors
            RucaptchaError::BuildHttpClient(_)
            | RucaptchaError::ParseResponse(_)
            | RucaptchaError::UnsupportedTask(_) => false,
        }
    }

    fn should_retry_operation(&self) -> bool {
        match self {
            // HTTP errors - retry the operation
            RucaptchaError::HttpRequest(_) => true,
            // Timeouts - the task expired but a fresh attempt might work
            RucaptchaError::SolutionTimeout { .. } => true,
            // API errors have their own logic
            RucaptchaError::Api(error) => error.error_code.should_retry_operation(),
            // Configuration errors - won't work until fixed
            RucaptchaError::BuildHttpClient(_)
            | RucaptchaError::ParseResponse(_)
            | RucaptchaError::UnsupportedTask(_) => false,
        }
    }
}

/// RuCaptcha API error codes
///
/// Based on <https://rucaptcha.com/api-docs/error-codes>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RucaptchaErrorCode {
    // === Retryable Errors (Temporary Issues) ===
    /// Your bid is too low or queue is too long
    #[serde(rename = "ERROR_NO_SLOT_AVAILABLE")]
    NoSlotAvailable,

    /// You have no funds in your account
    #[serde(rename = "ERROR_ZERO_BALANCE")]
    ZeroBalance,

    /// Unable to solve - workers failed; cost refunded
    #[serde(rename = "ERROR_CAPTCHA_UNSOLVABLE")]
    CaptchaUnsolvable,

    // === Permanent Errors (Configuration/Validation Issues) ===
    /// API key is incorrect
    #[serde(rename = "ERROR_KEY_DOES_NOT_EXIST")]
    KeyDoesNotExist,

    /// Image file size is under 100 bytes
    #[serde(rename = "ERROR_ZERO_CAPTCHA_FILESIZE")]
    ZeroCaptchaFilesize,

    /// Image exceeds 100 KB or 600 pixels on any side
    #[serde(rename = "ERROR_TOO_BIG_CAPTCHA_FILESIZE")]
    TooBigCaptchaFilesize,

    /// Website URL parameter missing or malformed
    #[serde(rename = "ERROR_PAGEURL")]
    PageUrl,

    /// Request from IP not in your trusted IP list
    #[serde(rename = "ERROR_IP_NOT_ALLOWED")]
    IpNotAllowed,

    /// 100% Recognition enabled; max attempts reached without min matches
    #[serde(rename = "ERROR_BAD_DUPLICATES")]
    BadDuplicates,

    /// API method doesn't exist
    #[serde(rename = "ERROR_NO_SUCH_METHOD")]
    NoSuchMethod,

    /// Image format invalid, corrupt, or wrong size
    #[serde(rename = "ERROR_IMAGE_TYPE_NOT_SUPPORTED")]
    ImageTypeNotSupported,

    /// Invalid task ID specified
    #[serde(rename = "ERROR_NO_SUCH_CAPCHA_ID")]
    NoSuchCaptchaId,

    /// IP address blocked due to API misuse
    #[serde(rename = "ERROR_IP_BLOCKED")]
    IpBlocked,

    /// Task property missing from createTask call
    #[serde(rename = "ERROR_TASK_ABSENT")]
    TaskAbsent,

    /// Task type unsupported or type property error
    #[serde(rename = "ERROR_TASK_NOT_SUPPORTED")]
    TaskNotSupported,

    /// Invalid sitekey value
    #[serde(rename = "ERROR_RECAPTCHA_INVALID_SITEKEY")]
    RecaptchaInvalidSitekey,

    /// Account blocked for API misuse
    #[serde(rename = "ERROR_ACCOUNT_SUSPENDED")]
    AccountSuspended,

    /// Required parameters missing or incorrectly formatted
    #[serde(rename = "ERROR_BAD_PARAMETERS")]
    BadParameters,

    /// Image instructions contain unsupported file type or exceed limits
    #[serde(rename = "ERROR_BAD_IMGINSTRUCTIONS")]
    BadImgInstructions,

    /// Invalid proxy parameters or connection failure
    #[serde(rename = "ERROR_BAD_PROXY")]
    BadProxy,

    /// Fallback for new API errors not in documentation
    #[serde(other)]
    Unknown,
}

impl RucaptchaErrorCode {
    /// Returns true if the same task should be retried
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::NoSlotAvailable)
    }

    /// Returns true if a fresh solve operation (new task) might succeed
    pub fn should_retry_operation(&self) -> bool {
        match self {
            // Transient server errors - retry
            Self::NoSlotAvailable => true,

            // Task-specific failures - fresh attempt might work
            Self::CaptchaUnsolvable => true,

            // Account issues - need to fix before retry
            Self::ZeroBalance
            | Self::KeyDoesNotExist
            | Self::IpNotAllowed
            | Self::IpBlocked
            | Self::AccountSuspended => false,

            // Configuration/validation issues - won't work until fixed
            Self::ZeroCaptchaFilesize
            | Self::TooBigCaptchaFilesize
            | Self::PageUrl
            | Self::BadDuplicates
            | Self::NoSuchMethod
            | Self::ImageTypeNotSupported
            | Self::NoSuchCaptchaId
            | Self::TaskAbsent
            | Self::TaskNotSupported
            | Self::RecaptchaInvalidSitekey
            | Self::BadParameters
            | Self::BadImgInstructions
            | Self::BadProxy => false,

            // Unknown errors - conservative: don't retry
            Self::Unknown => false,
        }
    }
}

impl fmt::Display for RucaptchaErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSlotAvailable => write!(f, "No slot available"),
            Self::ZeroBalance => write!(f, "Zero balance"),
            Self::CaptchaUnsolvable => write!(f, "Captcha unsolvable"),
            Self::KeyDoesNotExist => write!(f, "Key does not exist"),
            Self::ZeroCaptchaFilesize => write!(f, "Zero captcha filesize"),
            Self::TooBigCaptchaFilesize => write!(f, "Too big captcha filesize"),
            Self::PageUrl => write!(f, "Invalid page URL"),
            Self::IpNotAllowed => write!(f, "IP not allowed"),
            Self::BadDuplicates => write!(f, "Bad duplicates"),
            Self::NoSuchMethod => write!(f, "No such method"),
            Self::ImageTypeNotSupported => write!(f, "Image type not supported"),
            Self::NoSuchCaptchaId => write!(f, "No such captcha ID"),
            Self::IpBlocked => write!(f, "IP blocked"),
            Self::TaskAbsent => write!(f, "Task absent"),
            Self::TaskNotSupported => write!(f, "Task not supported"),
            Self::RecaptchaInvalidSitekey => write!(f, "Invalid reCAPTCHA sitekey"),
            Self::AccountSuspended => write!(f, "Account suspended"),
            Self::BadParameters => write!(f, "Bad parameters"),
            Self::BadImgInstructions => write!(f, "Bad image instructions"),
            Self::BadProxy => write!(f, "Bad proxy"),
            Self::Unknown => write!(f, "Unknown error"),
        }
    }
}

/// RuCaptcha API error response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RucaptchaApiError {
    pub error_id: u32,
    pub error_code: RucaptchaErrorCode,
    #[serde(default, alias = "errorDescription")]
    pub error_description: Option<String>,
}

impl fmt::Display for RucaptchaApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RuCaptcha Error [{}]: {} - {}",
            self.error_id,
            self.error_code,
            self.error_description
                .clone()
                .unwrap_or_else(|| "No description".to_string())
        )
    }
}

impl std::error::Error for RucaptchaApiError {}

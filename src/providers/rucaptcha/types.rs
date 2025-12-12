//! Task and solution types for the RuCaptcha API.

use crate::proxy::RucaptchaProxyFields;
use crate::serde_helpers::{deserialize_string_or_number, serialize_string_as_number_if_possible};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

// ============================================================================
// Task Types
// ============================================================================

/// RuCaptcha task types for the API request
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum RucaptchaTask {
    // -------------------------------------------------------------------------
    // ReCaptcha V2
    // -------------------------------------------------------------------------
    /// ReCaptcha V2 using service's built-in proxy
    RecaptchaV2TaskProxyless {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(
            rename = "recaptchaDataSValue",
            skip_serializing_if = "Option::is_none"
        )]
        recaptcha_data_s_value: Option<String>,
        #[serde(rename = "isInvisible", skip_serializing_if = "Option::is_none")]
        is_invisible: Option<bool>,
        #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
        user_agent: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cookies: Option<String>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
    },

    /// ReCaptcha V2 with custom proxy
    RecaptchaV2Task {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(
            rename = "recaptchaDataSValue",
            skip_serializing_if = "Option::is_none"
        )]
        recaptcha_data_s_value: Option<String>,
        #[serde(rename = "isInvisible", skip_serializing_if = "Option::is_none")]
        is_invisible: Option<bool>,
        #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
        user_agent: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cookies: Option<String>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
        #[serde(flatten)]
        proxy: RucaptchaProxyFields,
    },

    // -------------------------------------------------------------------------
    // ReCaptcha V2 Enterprise
    // -------------------------------------------------------------------------
    /// ReCaptcha V2 Enterprise using service's built-in proxy
    RecaptchaV2EnterpriseTaskProxyless {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(rename = "enterprisePayload", skip_serializing_if = "Option::is_none")]
        enterprise_payload: Option<serde_json::Value>,
        #[serde(rename = "isInvisible", skip_serializing_if = "Option::is_none")]
        is_invisible: Option<bool>,
        #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
        user_agent: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cookies: Option<String>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
    },

    /// ReCaptcha V2 Enterprise with custom proxy
    RecaptchaV2EnterpriseTask {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(rename = "enterprisePayload", skip_serializing_if = "Option::is_none")]
        enterprise_payload: Option<serde_json::Value>,
        #[serde(rename = "isInvisible", skip_serializing_if = "Option::is_none")]
        is_invisible: Option<bool>,
        #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
        user_agent: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cookies: Option<String>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
        #[serde(flatten)]
        proxy: RucaptchaProxyFields,
    },

    // -------------------------------------------------------------------------
    // ReCaptcha V3
    // -------------------------------------------------------------------------
    /// ReCaptcha V3 using service's built-in proxy
    RecaptchaV3TaskProxyless {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(rename = "minScore")]
        min_score: f32,
        #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
        page_action: Option<String>,
        #[serde(rename = "isEnterprise", skip_serializing_if = "Option::is_none")]
        is_enterprise: Option<bool>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
    },

    // -------------------------------------------------------------------------
    // Cloudflare Turnstile
    // -------------------------------------------------------------------------
    /// Cloudflare Turnstile using service's built-in proxy
    TurnstileTaskProxyless {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pagedata: Option<String>,
    },

    /// Cloudflare Turnstile with custom proxy
    TurnstileTask {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pagedata: Option<String>,
        #[serde(flatten)]
        proxy: RucaptchaProxyFields,
    },
}

impl Display for RucaptchaTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecaptchaV2TaskProxyless { .. } => write!(f, "ReCaptchaV2"),
            Self::RecaptchaV2Task { .. } => write!(f, "ReCaptchaV2"),
            Self::RecaptchaV2EnterpriseTaskProxyless { .. } => write!(f, "ReCaptchaV2Enterprise"),
            Self::RecaptchaV2EnterpriseTask { .. } => write!(f, "ReCaptchaV2Enterprise"),
            Self::RecaptchaV3TaskProxyless { is_enterprise: Some(true), .. } => {
                write!(f, "ReCaptchaV3Enterprise")
            }
            Self::RecaptchaV3TaskProxyless { .. } => write!(f, "ReCaptchaV3"),
            Self::TurnstileTaskProxyless { .. } => write!(f, "Turnstile"),
            Self::TurnstileTask { .. } => write!(f, "Turnstile"),
        }
    }
}

// ============================================================================
// Solution Types
// ============================================================================

// Re-export shared solution types for convenience
pub use crate::solutions::{ReCaptchaSolution, TurnstileSolution};

/// RuCaptcha solution types
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RucaptchaSolution {
    /// ReCaptcha solution (V2 or V3)
    ReCaptcha(ReCaptchaSolution),
    /// Turnstile solution
    Turnstile(TurnstileSolution),
}

impl RucaptchaSolution {
    /// Try to extract ReCaptcha solution (returns reference)
    pub fn as_recaptcha(&self) -> Option<&ReCaptchaSolution> {
        match self {
            Self::ReCaptcha(solution) => Some(solution),
            _ => None,
        }
    }

    /// Try to extract ReCaptcha solution (consumes self)
    ///
    /// Returns `Ok(solution)` if this is a ReCaptcha solution, or `Err(self)` otherwise.
    pub fn try_into_recaptcha(self) -> Result<ReCaptchaSolution, Self> {
        match self {
            Self::ReCaptcha(solution) => Ok(solution),
            other => Err(other),
        }
    }

    /// Extract ReCaptcha solution, panics if not ReCaptcha
    ///
    /// # Panics
    /// Panics if the solution is not a ReCaptcha solution.
    /// Use `try_into_recaptcha()` for a non-panicking alternative.
    pub fn into_recaptcha(self) -> ReCaptchaSolution {
        self.try_into_recaptcha()
            .expect("Expected ReCaptcha solution")
    }

    /// Try to extract Turnstile solution (returns reference)
    pub fn as_turnstile(&self) -> Option<&TurnstileSolution> {
        match self {
            Self::Turnstile(solution) => Some(solution),
            _ => None,
        }
    }

    /// Try to extract Turnstile solution (consumes self)
    ///
    /// Returns `Ok(solution)` if this is a Turnstile solution, or `Err(self)` otherwise.
    pub fn try_into_turnstile(self) -> Result<TurnstileSolution, Self> {
        match self {
            Self::Turnstile(solution) => Ok(solution),
            other => Err(other),
        }
    }

    /// Extract Turnstile solution, panics if not Turnstile
    ///
    /// # Panics
    /// Panics if the solution is not a Turnstile solution.
    /// Use `try_into_turnstile()` for a non-panicking alternative.
    pub fn into_turnstile(self) -> TurnstileSolution {
        self.try_into_turnstile()
            .expect("Expected Turnstile solution")
    }
}

// ============================================================================
// Internal Types (Request/Response)
// ============================================================================

/// Response data from RuCaptcha createTask endpoint (success case)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateTaskData {
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub task_id: String,
}

/// Response data from RuCaptcha getTaskResult endpoint (success case)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetTaskData<T> {
    #[allow(dead_code)]
    pub status: String,
    pub solution: Option<T>,
}

/// Request payload for creating a new task
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateTaskRequest<'a> {
    pub(crate) client_key: &'a str,
    pub(crate) task: &'a RucaptchaTask,
}

/// Request payload for getting task result
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetTaskResultRequest<'a> {
    pub(crate) client_key: &'a str,
    #[serde(serialize_with = "serialize_string_as_number_if_possible")]
    pub(crate) task_id: &'a str,
}

// ============================================================================
// From implementations for shared task types
// ============================================================================

impl From<crate::tasks::ReCaptchaV2> for RucaptchaTask {
    fn from(task: crate::tasks::ReCaptchaV2) -> Self {
        let is_invisible = if task.is_invisible { Some(true) } else { None };
        let enterprise_payload = task.enterprise_payload.map(|p| serde_json::to_value(p).unwrap_or_default());

        match (task.is_enterprise, task.proxy) {
            // Enterprise with proxy
            (true, Some(proxy)) => Self::RecaptchaV2EnterpriseTask {
                website_url: task.website_url,
                website_key: task.website_key,
                enterprise_payload,
                is_invisible,
                user_agent: task.user_agent,
                cookies: task.cookies,
                api_domain: task.api_domain,
                proxy: proxy.into_rucaptcha_fields(),
            },
            // Enterprise without proxy
            (true, None) => Self::RecaptchaV2EnterpriseTaskProxyless {
                website_url: task.website_url,
                website_key: task.website_key,
                enterprise_payload,
                is_invisible,
                user_agent: task.user_agent,
                cookies: task.cookies,
                api_domain: task.api_domain,
            },
            // Standard with proxy
            (false, Some(proxy)) => Self::RecaptchaV2Task {
                website_url: task.website_url,
                website_key: task.website_key,
                recaptcha_data_s_value: task.recaptcha_data_s_value,
                is_invisible,
                user_agent: task.user_agent,
                cookies: task.cookies,
                api_domain: task.api_domain,
                proxy: proxy.into_rucaptcha_fields(),
            },
            // Standard without proxy
            (false, None) => Self::RecaptchaV2TaskProxyless {
                website_url: task.website_url,
                website_key: task.website_key,
                recaptcha_data_s_value: task.recaptcha_data_s_value,
                is_invisible,
                user_agent: task.user_agent,
                cookies: task.cookies,
                api_domain: task.api_domain,
            },
        }
    }
}

impl From<crate::tasks::ReCaptchaV3> for RucaptchaTask {
    fn from(task: crate::tasks::ReCaptchaV3) -> Self {
        let is_enterprise = if task.is_enterprise { Some(true) } else { None };
        // RuCaptcha V3 uses min_score, default to 0.9 if not specified
        let min_score = task.min_score.unwrap_or(0.9);

        Self::RecaptchaV3TaskProxyless {
            website_url: task.website_url,
            website_key: task.website_key,
            min_score,
            page_action: task.page_action,
            is_enterprise,
            api_domain: task.api_domain,
        }
    }
}

impl From<crate::tasks::Turnstile> for RucaptchaTask {
    fn from(task: crate::tasks::Turnstile) -> Self {
        match task.proxy {
            Some(proxy) => Self::TurnstileTask {
                website_url: task.website_url,
                website_key: task.website_key,
                action: task.action,
                data: task.cdata,
                pagedata: task.pagedata,
                proxy: proxy.into_rucaptcha_fields(),
            },
            None => Self::TurnstileTaskProxyless {
                website_url: task.website_url,
                website_key: task.website_key,
                action: task.action,
                data: task.cdata,
                pagedata: task.pagedata,
            },
        }
    }
}

impl TryFrom<crate::tasks::CloudflareChallenge> for RucaptchaTask {
    type Error = crate::errors::UnsupportedTaskError;

    /// Attempt to convert a CloudflareChallenge task to RuCaptcha format.
    ///
    /// # Errors
    ///
    /// Always returns an error because CloudflareChallenge is not supported by RuCaptcha.
    /// This task type is only available with Capsolver.
    fn try_from(_task: crate::tasks::CloudflareChallenge) -> Result<Self, Self::Error> {
        Err(crate::errors::UnsupportedTaskError::new(
            "CloudflareChallenge",
            "RuCaptcha",
        ))
    }
}

impl TryFrom<crate::tasks::CaptchaTask> for RucaptchaTask {
    type Error = crate::errors::UnsupportedTaskError;

    fn try_from(task: crate::tasks::CaptchaTask) -> Result<Self, Self::Error> {
        match task {
            crate::tasks::CaptchaTask::ReCaptchaV2(t) => Ok(t.into()),
            crate::tasks::CaptchaTask::ReCaptchaV3(t) => Ok(t.into()),
            crate::tasks::CaptchaTask::Turnstile(t) => Ok(t.into()),
            crate::tasks::CaptchaTask::CloudflareChallenge(t) => t.try_into(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::{ProxyConfig, ProxyType};
    use crate::tasks::{CloudflareChallenge, ReCaptchaV2, ReCaptchaV3, Turnstile};

    #[test]
    fn test_recaptcha_v2_serialization() {
        let task: RucaptchaTask = ReCaptchaV2::new("https://example.com", "site-key").into();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("RecaptchaV2TaskProxyless"));
        assert!(json.contains("websiteURL"));
        assert!(json.contains("websiteKey"));
    }

    #[test]
    fn test_recaptcha_v2_invisible_serialization() {
        let task: RucaptchaTask = ReCaptchaV2::new("https://example.com", "site-key")
            .invisible()
            .into();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("isInvisible"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_recaptcha_v2_with_proxy_serialization() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080).with_auth("user", "pass");
        let task: RucaptchaTask = ReCaptchaV2::new("https://example.com", "site-key")
            .with_proxy(proxy)
            .into();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("RecaptchaV2Task"));
        assert!(json.contains("proxyType"));
        assert!(json.contains("proxyAddress"));
        assert!(json.contains("proxyPort"));
        assert!(json.contains("proxyLogin"));
        assert!(json.contains("proxyPassword"));
    }

    #[test]
    fn test_recaptcha_v3_serialization() {
        let task: RucaptchaTask = ReCaptchaV3::new("https://example.com", "site-key")
            .with_min_score(0.9)
            .into();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("RecaptchaV3TaskProxyless"));
        assert!(json.contains("minScore"));
        assert!(json.contains("0.9"));
    }

    #[test]
    fn test_recaptcha_v3_with_action_serialization() {
        let task: RucaptchaTask = ReCaptchaV3::new("https://example.com", "site-key")
            .with_action("submit")
            .with_min_score(0.7)
            .into();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("pageAction"));
        assert!(json.contains("submit"));
    }

    #[test]
    fn test_turnstile_serialization() {
        let task: RucaptchaTask = Turnstile::new("https://example.com", "site-key").into();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("TurnstileTaskProxyless"));
        assert!(!json.contains("action"));
    }

    #[test]
    fn test_turnstile_with_metadata_serialization() {
        let task: RucaptchaTask = Turnstile::new("https://example.com", "site-key")
            .with_action("login")
            .with_cdata("cdata")
            .into();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("action"));
        assert!(json.contains("login"));
        assert!(json.contains("data"));
    }

    #[test]
    fn test_recaptcha_solution_deserialization() {
        let json = r#"{
            "gRecaptchaResponse": "token-value",
            "token": "token-value"
        }"#;
        let solution: ReCaptchaSolution = serde_json::from_str(json).unwrap();
        assert_eq!(solution.token(), "token-value");
    }

    #[test]
    fn test_turnstile_solution_deserialization() {
        let json = r#"{
            "token": "turnstile-token"
        }"#;
        let solution: TurnstileSolution = serde_json::from_str(json).unwrap();
        assert_eq!(solution.token(), "turnstile-token");
    }

    #[test]
    fn test_task_display() {
        let task: RucaptchaTask = ReCaptchaV2::new("url", "key").into();
        assert_eq!(task.to_string(), "ReCaptchaV2");

        let task: RucaptchaTask = ReCaptchaV3::new("url", "key").into();
        assert_eq!(task.to_string(), "ReCaptchaV3");

        let task: RucaptchaTask = ReCaptchaV3::new("url", "key").enterprise().into();
        assert_eq!(task.to_string(), "ReCaptchaV3Enterprise");

        let task: RucaptchaTask = Turnstile::new("url", "key").into();
        assert_eq!(task.to_string(), "Turnstile");
    }

    #[test]
    fn test_proxy_config() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        assert_eq!(proxy.proxy_type, ProxyType::Http);
        assert_eq!(proxy.address, "192.168.1.1");
        assert_eq!(proxy.port, 8080);
        assert!(proxy.login.is_none());

        let proxy = proxy.with_auth("user", "pass");
        assert_eq!(proxy.login.as_deref(), Some("user"));
        assert_eq!(proxy.password.as_deref(), Some("pass"));
    }

    #[test]
    fn test_create_task_data_numeric_task_id() {
        // RuCaptcha returns taskId as a number, not a string
        let json = r#"{"errorId":0,"taskId":54137240716}"#;
        let data: CreateTaskData = serde_json::from_str(json).unwrap();
        assert_eq!(data.task_id, "54137240716");
    }

    #[test]
    fn test_create_task_data_string_task_id() {
        // But we should also accept string format
        let json = r#"{"errorId":0,"taskId":"54137240716"}"#;
        let data: CreateTaskData = serde_json::from_str(json).unwrap();
        assert_eq!(data.task_id, "54137240716");
    }

    #[test]
    fn test_get_task_result_request_numeric_serialization() {
        // When task_id is numeric, it should serialize as a number
        let request = GetTaskResultRequest {
            client_key: "test-key",
            task_id: "54137240716",
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"taskId\":54137240716"));
        assert!(!json.contains("\"taskId\":\"54137240716\"")); // Should NOT be a string
    }

    #[test]
    fn test_get_task_result_request_string_serialization() {
        // When task_id is not numeric, it should serialize as a string
        let request = GetTaskResultRequest {
            client_key: "test-key",
            task_id: "abc-123-def",
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"taskId\":\"abc-123-def\""));
    }

    #[test]
    fn test_from_shared_recaptcha_v2() {
        let task = ReCaptchaV2::new("https://example.com", "key");
        let rucaptcha_task: RucaptchaTask = task.into();
        let json = serde_json::to_string(&rucaptcha_task).unwrap();
        assert!(json.contains("RecaptchaV2TaskProxyless"));
    }

    #[test]
    fn test_from_shared_recaptcha_v2_with_proxy() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        let task = ReCaptchaV2::new("https://example.com", "key")
            .with_proxy(proxy);
        let rucaptcha_task: RucaptchaTask = task.into();
        let json = serde_json::to_string(&rucaptcha_task).unwrap();
        assert!(json.contains("RecaptchaV2Task"));
        assert!(json.contains("proxyAddress"));
    }

    #[test]
    fn test_from_shared_turnstile() {
        let task = Turnstile::new("https://example.com", "key")
            .with_action("login");
        let rucaptcha_task: RucaptchaTask = task.into();
        let json = serde_json::to_string(&rucaptcha_task).unwrap();
        assert!(json.contains("TurnstileTaskProxyless"));
        assert!(json.contains("\"action\":\"login\""));
    }

    #[test]
    fn test_cloudflare_challenge_unsupported() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        let task = CloudflareChallenge::new("https://example.com", proxy);
        let result: Result<RucaptchaTask, _> = task.try_into();

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.task_type, "CloudflareChallenge");
        assert_eq!(error.provider, "RuCaptcha");
        assert!(error.to_string().contains("not supported by RuCaptcha"));
    }
}
//! Task and solution types for the RuCaptcha API.

use crate::proxy::{ProxyConfig, ProxyType};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Display;

/// Deserialize a value that can be either a string or a number into a String
fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        _ => Err(D::Error::custom("expected string or number")),
    }
}

/// Serialize a string as a number if it's numeric, otherwise as a string
fn serialize_string_as_number_if_possible<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // Try to parse as u64 first (most common for task IDs)
    if let Ok(n) = value.parse::<u64>() {
        return serializer.serialize_u64(n);
    }
    // Fall back to string
    serializer.serialize_str(value)
}

/// Serialize ProxyType field for serde (RuCaptcha uses lowercase)
fn serialize_proxy_type_field<S>(proxy_type: &ProxyType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let type_str = match proxy_type {
        // RuCaptcha only supports http, socks4, socks5 (not https separately)
        ProxyType::Http | ProxyType::Https => "http",
        ProxyType::Socks4 => "socks4",
        ProxyType::Socks5 => "socks5",
    };
    serializer.serialize_str(type_str)
}

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
        // Proxy fields
        #[serde(rename = "proxyType", serialize_with = "serialize_proxy_type_field")]
        proxy_type: ProxyType,
        #[serde(rename = "proxyAddress")]
        proxy_address: String,
        #[serde(rename = "proxyPort")]
        proxy_port: u16,
        #[serde(rename = "proxyLogin", skip_serializing_if = "Option::is_none")]
        proxy_login: Option<String>,
        #[serde(rename = "proxyPassword", skip_serializing_if = "Option::is_none")]
        proxy_password: Option<String>,
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
        // Proxy fields
        #[serde(rename = "proxyType", serialize_with = "serialize_proxy_type_field")]
        proxy_type: ProxyType,
        #[serde(rename = "proxyAddress")]
        proxy_address: String,
        #[serde(rename = "proxyPort")]
        proxy_port: u16,
        #[serde(rename = "proxyLogin", skip_serializing_if = "Option::is_none")]
        proxy_login: Option<String>,
        #[serde(rename = "proxyPassword", skip_serializing_if = "Option::is_none")]
        proxy_password: Option<String>,
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
        // Proxy fields
        #[serde(rename = "proxyType", serialize_with = "serialize_proxy_type_field")]
        proxy_type: ProxyType,
        #[serde(rename = "proxyAddress")]
        proxy_address: String,
        #[serde(rename = "proxyPort")]
        proxy_port: u16,
        #[serde(rename = "proxyLogin", skip_serializing_if = "Option::is_none")]
        proxy_login: Option<String>,
        #[serde(rename = "proxyPassword", skip_serializing_if = "Option::is_none")]
        proxy_password: Option<String>,
    },
}

/// Turnstile metadata for challenge pages
#[derive(Debug, Clone, Default)]
pub struct TurnstileMetadata {
    /// Value from turnstile.render action parameter
    pub action: Option<String>,
    /// Value from turnstile.render cData parameter
    pub data: Option<String>,
    /// Value from turnstile.render chlPageData parameter
    pub pagedata: Option<String>,
}

impl RucaptchaTask {
    // -------------------------------------------------------------------------
    // ReCaptcha V2 Constructors
    // -------------------------------------------------------------------------

    /// Create a ReCaptcha V2 task (proxyless)
    pub fn recaptcha_v2(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
    ) -> Self {
        Self::RecaptchaV2TaskProxyless {
            website_url: website_url.into(),
            website_key: website_key.into(),
            recaptcha_data_s_value: None,
            is_invisible: None,
            user_agent: None,
            cookies: None,
            api_domain: None,
        }
    }

    /// Create an invisible ReCaptcha V2 task (proxyless)
    pub fn recaptcha_v2_invisible(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
    ) -> Self {
        Self::RecaptchaV2TaskProxyless {
            website_url: website_url.into(),
            website_key: website_key.into(),
            recaptcha_data_s_value: None,
            is_invisible: Some(true),
            user_agent: None,
            cookies: None,
            api_domain: None,
        }
    }

    /// Create a ReCaptcha V2 task with proxy
    pub fn recaptcha_v2_with_proxy(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        proxy: ProxyConfig,
    ) -> Self {
        Self::RecaptchaV2Task {
            website_url: website_url.into(),
            website_key: website_key.into(),
            recaptcha_data_s_value: None,
            is_invisible: None,
            user_agent: None,
            cookies: None,
            api_domain: None,
            proxy_type: proxy.proxy_type,
            proxy_address: proxy.address,
            proxy_port: proxy.port,
            proxy_login: proxy.login,
            proxy_password: proxy.password,
        }
    }

    // -------------------------------------------------------------------------
    // ReCaptcha V2 Enterprise Constructors
    // -------------------------------------------------------------------------

    /// Create a ReCaptcha V2 Enterprise task (proxyless)
    pub fn recaptcha_v2_enterprise(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
    ) -> Self {
        Self::RecaptchaV2EnterpriseTaskProxyless {
            website_url: website_url.into(),
            website_key: website_key.into(),
            enterprise_payload: None,
            is_invisible: None,
            user_agent: None,
            cookies: None,
            api_domain: None,
        }
    }

    /// Create a ReCaptcha V2 Enterprise task with proxy
    pub fn recaptcha_v2_enterprise_with_proxy(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        proxy: ProxyConfig,
    ) -> Self {
        Self::RecaptchaV2EnterpriseTask {
            website_url: website_url.into(),
            website_key: website_key.into(),
            enterprise_payload: None,
            is_invisible: None,
            user_agent: None,
            cookies: None,
            api_domain: None,
            proxy_type: proxy.proxy_type,
            proxy_address: proxy.address,
            proxy_port: proxy.port,
            proxy_login: proxy.login,
            proxy_password: proxy.password,
        }
    }

    // -------------------------------------------------------------------------
    // ReCaptcha V3 Constructors
    // -------------------------------------------------------------------------

    /// Create a ReCaptcha V3 task (proxyless)
    ///
    /// # Arguments
    /// * `website_url` - Full URL of the target page
    /// * `website_key` - The reCAPTCHA sitekey
    /// * `min_score` - Minimum score threshold (0.3, 0.7, or 0.9)
    pub fn recaptcha_v3(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        min_score: f32,
    ) -> Self {
        Self::RecaptchaV3TaskProxyless {
            website_url: website_url.into(),
            website_key: website_key.into(),
            min_score,
            page_action: None,
            is_enterprise: None,
            api_domain: None,
        }
    }

    /// Create a ReCaptcha V3 task with action (proxyless)
    pub fn recaptcha_v3_with_action(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        min_score: f32,
        page_action: impl Into<String>,
    ) -> Self {
        Self::RecaptchaV3TaskProxyless {
            website_url: website_url.into(),
            website_key: website_key.into(),
            min_score,
            page_action: Some(page_action.into()),
            is_enterprise: None,
            api_domain: None,
        }
    }

    /// Create a ReCaptcha V3 Enterprise task (proxyless)
    pub fn recaptcha_v3_enterprise(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        min_score: f32,
    ) -> Self {
        Self::RecaptchaV3TaskProxyless {
            website_url: website_url.into(),
            website_key: website_key.into(),
            min_score,
            page_action: None,
            is_enterprise: Some(true),
            api_domain: None,
        }
    }

    // -------------------------------------------------------------------------
    // Turnstile Constructors
    // -------------------------------------------------------------------------

    /// Create a Turnstile task (proxyless)
    pub fn turnstile(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
    ) -> Self {
        Self::TurnstileTaskProxyless {
            website_url: website_url.into(),
            website_key: website_key.into(),
            action: None,
            data: None,
            pagedata: None,
        }
    }

    /// Create a Turnstile task with metadata (proxyless)
    pub fn turnstile_with_metadata(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        metadata: TurnstileMetadata,
    ) -> Self {
        Self::TurnstileTaskProxyless {
            website_url: website_url.into(),
            website_key: website_key.into(),
            action: metadata.action,
            data: metadata.data,
            pagedata: metadata.pagedata,
        }
    }

    /// Create a Turnstile task with proxy
    pub fn turnstile_with_proxy(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        proxy: ProxyConfig,
    ) -> Self {
        Self::TurnstileTask {
            website_url: website_url.into(),
            website_key: website_key.into(),
            action: None,
            data: None,
            pagedata: None,
            proxy_type: proxy.proxy_type,
            proxy_address: proxy.address,
            proxy_port: proxy.port,
            proxy_login: proxy.login,
            proxy_password: proxy.password,
        }
    }
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

/// ReCaptcha solution (V2 and V3)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReCaptchaSolution {
    /// The reCAPTCHA token
    #[serde(rename = "gRecaptchaResponse")]
    pub g_recaptcha_response: String,
    /// Alias for gRecaptchaResponse
    #[serde(default)]
    pub token: Option<String>,
}

impl ReCaptchaSolution {
    /// Get the reCAPTCHA token
    pub fn token(&self) -> &str {
        &self.g_recaptcha_response
    }
}

/// Turnstile captcha solution
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnstileSolution {
    /// The solved Turnstile token
    pub token: String,
}

impl TurnstileSolution {
    /// Get the Turnstile token
    pub fn token(&self) -> &str {
        &self.token
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recaptcha_v2_serialization() {
        let task = RucaptchaTask::recaptcha_v2("https://example.com", "site-key");
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("RecaptchaV2TaskProxyless"));
        assert!(json.contains("websiteURL"));
        assert!(json.contains("websiteKey"));
    }

    #[test]
    fn test_recaptcha_v2_invisible_serialization() {
        let task = RucaptchaTask::recaptcha_v2_invisible("https://example.com", "site-key");
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("isInvisible"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_recaptcha_v2_with_proxy_serialization() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080).with_auth("user", "pass");
        let task = RucaptchaTask::recaptcha_v2_with_proxy("https://example.com", "site-key", proxy);
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
        let task = RucaptchaTask::recaptcha_v3("https://example.com", "site-key", 0.9);
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("RecaptchaV3TaskProxyless"));
        assert!(json.contains("minScore"));
        assert!(json.contains("0.9"));
    }

    #[test]
    fn test_recaptcha_v3_with_action_serialization() {
        let task = RucaptchaTask::recaptcha_v3_with_action(
            "https://example.com",
            "site-key",
            0.7,
            "submit",
        );
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("pageAction"));
        assert!(json.contains("submit"));
    }

    #[test]
    fn test_turnstile_serialization() {
        let task = RucaptchaTask::turnstile("https://example.com", "site-key");
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("TurnstileTaskProxyless"));
        assert!(!json.contains("action"));
    }

    #[test]
    fn test_turnstile_with_metadata_serialization() {
        let metadata = TurnstileMetadata {
            action: Some("login".to_string()),
            data: Some("cdata".to_string()),
            pagedata: None,
        };
        let task = RucaptchaTask::turnstile_with_metadata("https://example.com", "site-key", metadata);
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
        assert_eq!(
            RucaptchaTask::recaptcha_v2("url", "key").to_string(),
            "ReCaptchaV2"
        );
        assert_eq!(
            RucaptchaTask::recaptcha_v3("url", "key", 0.9).to_string(),
            "ReCaptchaV3"
        );
        assert_eq!(
            RucaptchaTask::recaptcha_v3_enterprise("url", "key", 0.9).to_string(),
            "ReCaptchaV3Enterprise"
        );
        assert_eq!(
            RucaptchaTask::turnstile("url", "key").to_string(),
            "Turnstile"
        );
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
}
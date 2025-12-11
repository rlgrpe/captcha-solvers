use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

// ============================================================================
// Task Types
// ============================================================================

/// Capsolver task types for the API request
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum CapsolverTask {
    // -------------------------------------------------------------------------
    // ReCaptcha V2
    // -------------------------------------------------------------------------
    /// ReCaptcha V2 using server's built-in proxy
    ReCaptchaV2TaskProxyLess {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
        page_action: Option<String>,
        #[serde(
            rename = "recaptchaDataSValue",
            skip_serializing_if = "Option::is_none"
        )]
        recaptcha_data_s_value: Option<String>,
        #[serde(rename = "isInvisible", skip_serializing_if = "Option::is_none")]
        is_invisible: Option<bool>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
    },

    /// ReCaptcha V2 Enterprise requiring custom proxy
    ReCaptchaV2EnterpriseTask {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        proxy: Option<String>,
        #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
        page_action: Option<String>,
        #[serde(rename = "enterprisePayload", skip_serializing_if = "Option::is_none")]
        enterprise_payload: Option<HashMap<String, serde_json::Value>>,
        #[serde(rename = "isInvisible", skip_serializing_if = "Option::is_none")]
        is_invisible: Option<bool>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
    },

    /// ReCaptcha V2 Enterprise using server's built-in proxy
    ReCaptchaV2EnterpriseTaskProxyLess {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
        page_action: Option<String>,
        #[serde(rename = "enterprisePayload", skip_serializing_if = "Option::is_none")]
        enterprise_payload: Option<HashMap<String, serde_json::Value>>,
        #[serde(rename = "isInvisible", skip_serializing_if = "Option::is_none")]
        is_invisible: Option<bool>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
    },

    // -------------------------------------------------------------------------
    // ReCaptcha V3
    // -------------------------------------------------------------------------
    /// ReCaptcha V3 requiring custom proxy
    ReCaptchaV3Task {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        proxy: Option<String>,
        #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
        page_action: Option<String>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
    },

    /// ReCaptcha V3 using server's built-in proxy
    ReCaptchaV3TaskProxyLess {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
        page_action: Option<String>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
    },

    /// ReCaptcha V3 Enterprise requiring custom proxy
    ReCaptchaV3EnterpriseTask {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        proxy: Option<String>,
        #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
        page_action: Option<String>,
        #[serde(rename = "enterprisePayload", skip_serializing_if = "Option::is_none")]
        enterprise_payload: Option<HashMap<String, serde_json::Value>>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
    },

    /// ReCaptcha V3 Enterprise using server's built-in proxy
    ReCaptchaV3EnterpriseTaskProxyLess {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
        page_action: Option<String>,
        #[serde(rename = "enterprisePayload", skip_serializing_if = "Option::is_none")]
        enterprise_payload: Option<HashMap<String, serde_json::Value>>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
    },

    // -------------------------------------------------------------------------
    // Cloudflare Turnstile
    // -------------------------------------------------------------------------
    /// Cloudflare Turnstile captcha (proxyless)
    AntiTurnstileTaskProxyLess {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<TurnstileMetadata>,
    },

    // -------------------------------------------------------------------------
    // Cloudflare Challenge
    // -------------------------------------------------------------------------
    /// Cloudflare Challenge (requires proxy)
    AntiCloudflareTask {
        #[serde(rename = "websiteURL")]
        website_url: String,
        /// Static or Sticky proxy (not Rotating)
        proxy: String,
        #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
        user_agent: Option<String>,
        /// Response HTML from target website (typically contains "Just a moment...")
        #[serde(skip_serializing_if = "Option::is_none")]
        html: Option<String>,
    },
}

/// Metadata for Turnstile captcha
#[derive(Debug, Clone, Serialize, Default)]
pub struct TurnstileMetadata {
    /// Value of the `data-action` attribute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// Value of the `data-cdata` attribute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdata: Option<String>,
}

impl CapsolverTask {
    // -------------------------------------------------------------------------
    // ReCaptcha V2 Constructors
    // -------------------------------------------------------------------------

    /// Create a ReCaptcha V2 task (proxyless)
    pub fn recaptcha_v2(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
    ) -> Self {
        Self::ReCaptchaV2TaskProxyLess {
            website_url: website_url.into(),
            website_key: website_key.into(),
            page_action: None,
            recaptcha_data_s_value: None,
            is_invisible: None,
            api_domain: None,
        }
    }

    /// Create an invisible ReCaptcha V2 task (proxyless)
    pub fn recaptcha_v2_invisible(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
    ) -> Self {
        Self::ReCaptchaV2TaskProxyLess {
            website_url: website_url.into(),
            website_key: website_key.into(),
            page_action: None,
            recaptcha_data_s_value: None,
            is_invisible: Some(true),
            api_domain: None,
        }
    }

    /// Create a ReCaptcha V2 Enterprise task (proxyless)
    pub fn recaptcha_v2_enterprise(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
    ) -> Self {
        Self::ReCaptchaV2EnterpriseTaskProxyLess {
            website_url: website_url.into(),
            website_key: website_key.into(),
            page_action: None,
            enterprise_payload: None,
            is_invisible: None,
            api_domain: None,
        }
    }

    /// Create a ReCaptcha V2 Enterprise task with proxy
    pub fn recaptcha_v2_enterprise_with_proxy(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        proxy: impl Into<String>,
    ) -> Self {
        Self::ReCaptchaV2EnterpriseTask {
            website_url: website_url.into(),
            website_key: website_key.into(),
            proxy: Some(proxy.into()),
            page_action: None,
            enterprise_payload: None,
            is_invisible: None,
            api_domain: None,
        }
    }

    // -------------------------------------------------------------------------
    // ReCaptcha V3 Constructors
    // -------------------------------------------------------------------------

    /// Create a ReCaptcha V3 task (proxyless)
    pub fn recaptcha_v3(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
    ) -> Self {
        Self::ReCaptchaV3TaskProxyLess {
            website_url: website_url.into(),
            website_key: website_key.into(),
            page_action: None,
            api_domain: None,
        }
    }

    /// Create a ReCaptcha V3 task with action (proxyless)
    pub fn recaptcha_v3_with_action(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        page_action: impl Into<String>,
    ) -> Self {
        Self::ReCaptchaV3TaskProxyLess {
            website_url: website_url.into(),
            website_key: website_key.into(),
            page_action: Some(page_action.into()),
            api_domain: None,
        }
    }

    /// Create a ReCaptcha V3 task with proxy
    pub fn recaptcha_v3_with_proxy(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        proxy: impl Into<String>,
    ) -> Self {
        Self::ReCaptchaV3Task {
            website_url: website_url.into(),
            website_key: website_key.into(),
            proxy: Some(proxy.into()),
            page_action: None,
            api_domain: None,
        }
    }

    /// Create a ReCaptcha V3 Enterprise task (proxyless)
    pub fn recaptcha_v3_enterprise(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
    ) -> Self {
        Self::ReCaptchaV3EnterpriseTaskProxyLess {
            website_url: website_url.into(),
            website_key: website_key.into(),
            page_action: None,
            enterprise_payload: None,
            api_domain: None,
        }
    }

    /// Create a ReCaptcha V3 Enterprise task with proxy
    pub fn recaptcha_v3_enterprise_with_proxy(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        proxy: impl Into<String>,
    ) -> Self {
        Self::ReCaptchaV3EnterpriseTask {
            website_url: website_url.into(),
            website_key: website_key.into(),
            proxy: Some(proxy.into()),
            page_action: None,
            enterprise_payload: None,
            api_domain: None,
        }
    }

    // -------------------------------------------------------------------------
    // Turnstile Constructors
    // -------------------------------------------------------------------------

    /// Create a Turnstile task
    pub fn turnstile(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
    ) -> Self {
        Self::AntiTurnstileTaskProxyLess {
            website_url: website_url.into(),
            website_key: website_key.into(),
            metadata: None,
        }
    }

    /// Create a Turnstile task with metadata
    pub fn turnstile_with_metadata(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        metadata: TurnstileMetadata,
    ) -> Self {
        Self::AntiTurnstileTaskProxyLess {
            website_url: website_url.into(),
            website_key: website_key.into(),
            metadata: Some(metadata),
        }
    }

    // -------------------------------------------------------------------------
    // Cloudflare Challenge Constructors
    // -------------------------------------------------------------------------

    /// Create a Cloudflare Challenge task (requires proxy)
    pub fn cloudflare_challenge(
        website_url: impl Into<String>,
        proxy: impl Into<String>,
    ) -> Self {
        Self::AntiCloudflareTask {
            website_url: website_url.into(),
            proxy: proxy.into(),
            user_agent: None,
            html: None,
        }
    }

    /// Create a Cloudflare Challenge task with user agent
    pub fn cloudflare_challenge_with_user_agent(
        website_url: impl Into<String>,
        proxy: impl Into<String>,
        user_agent: impl Into<String>,
    ) -> Self {
        Self::AntiCloudflareTask {
            website_url: website_url.into(),
            proxy: proxy.into(),
            user_agent: Some(user_agent.into()),
            html: None,
        }
    }
}

impl Display for CapsolverTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReCaptchaV2TaskProxyLess { .. } => write!(f, "ReCaptchaV2"),
            Self::ReCaptchaV2EnterpriseTask { .. } => write!(f, "ReCaptchaV2Enterprise"),
            Self::ReCaptchaV2EnterpriseTaskProxyLess { .. } => {
                write!(f, "ReCaptchaV2Enterprise")
            }
            Self::ReCaptchaV3Task { .. } => write!(f, "ReCaptchaV3"),
            Self::ReCaptchaV3TaskProxyLess { .. } => write!(f, "ReCaptchaV3"),
            Self::ReCaptchaV3EnterpriseTask { .. } => write!(f, "ReCaptchaV3Enterprise"),
            Self::ReCaptchaV3EnterpriseTaskProxyLess { .. } => {
                write!(f, "ReCaptchaV3Enterprise")
            }
            Self::AntiTurnstileTaskProxyLess { .. } => write!(f, "Turnstile"),
            Self::AntiCloudflareTask { .. } => write!(f, "CloudflareChallenge"),
        }
    }
}

// ============================================================================
// Solution Types
// ============================================================================

/// Capsolver solution types
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CapsolverSolution {
    /// ReCaptcha solution (V2 or V3)
    ReCaptcha(ReCaptchaSolution),
    /// Turnstile solution
    Turnstile(TurnstileSolution),
    /// Cloudflare Challenge solution
    CloudflareChallenge(CloudflareChallengeSolution),
}

impl CapsolverSolution {
    /// Try to extract ReCaptcha solution
    pub fn as_recaptcha(&self) -> Option<&ReCaptchaSolution> {
        match self {
            Self::ReCaptcha(solution) => Some(solution),
            _ => None,
        }
    }

    /// Extract ReCaptcha solution, panics if not ReCaptcha
    pub fn into_recaptcha(self) -> ReCaptchaSolution {
        match self {
            Self::ReCaptcha(solution) => solution,
            _ => panic!("Expected ReCaptcha solution"),
        }
    }

    /// Try to extract Turnstile solution
    pub fn as_turnstile(&self) -> Option<&TurnstileSolution> {
        match self {
            Self::Turnstile(solution) => Some(solution),
            _ => None,
        }
    }

    /// Extract Turnstile solution, panics if not Turnstile
    pub fn into_turnstile(self) -> TurnstileSolution {
        match self {
            Self::Turnstile(solution) => solution,
            _ => panic!("Expected Turnstile solution"),
        }
    }

    /// Try to extract Cloudflare Challenge solution
    pub fn as_cloudflare_challenge(&self) -> Option<&CloudflareChallengeSolution> {
        match self {
            Self::CloudflareChallenge(solution) => Some(solution),
            _ => None,
        }
    }

    /// Extract Cloudflare Challenge solution, panics if not CloudflareChallenge
    pub fn into_cloudflare_challenge(self) -> CloudflareChallengeSolution {
        match self {
            Self::CloudflareChallenge(solution) => solution,
            _ => panic!("Expected CloudflareChallenge solution"),
        }
    }
}

/// ReCaptcha solution (V2 and V3)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReCaptchaSolution {
    /// The reCAPTCHA token
    #[serde(rename = "gRecaptchaResponse")]
    pub g_recaptcha_response: String,
    /// User-Agent string used
    #[serde(default)]
    pub user_agent: Option<String>,
    /// Sec-Ch-Ua header value
    #[serde(default, rename = "secChUa")]
    pub sec_ch_ua: Option<String>,
    /// Token creation timestamp
    #[serde(default)]
    pub create_time: Option<u64>,
    /// Session cookie for V3 (when isSession is enabled)
    #[serde(default, rename = "recaptcha-ca-t")]
    pub recaptcha_ca_t: Option<String>,
    /// Cookie for some V2 websites
    #[serde(default, rename = "recaptcha-ca-e")]
    pub recaptcha_ca_e: Option<String>,
}

impl ReCaptchaSolution {
    /// Get the reCAPTCHA token
    pub fn token(&self) -> &str {
        &self.g_recaptcha_response
    }

    /// Get the session cookie (for V3 with isSession enabled)
    pub fn session_cookie(&self) -> Option<&str> {
        self.recaptcha_ca_t.as_deref()
    }
}

/// Turnstile captcha solution
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnstileSolution {
    /// The solved Turnstile token
    pub token: String,
    /// User-Agent string used
    #[serde(default)]
    pub user_agent: Option<String>,
}

impl TurnstileSolution {
    /// Get the Turnstile token
    pub fn token(&self) -> &str {
        &self.token
    }
}

/// Cloudflare Challenge solution
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudflareChallengeSolution {
    /// The cf_clearance cookie value
    pub token: String,
    /// Cookies map containing cf_clearance
    #[serde(default)]
    pub cookies: Option<HashMap<String, String>>,
    /// User-Agent string used (must match your requests)
    #[serde(default)]
    pub user_agent: Option<String>,
}

impl CloudflareChallengeSolution {
    /// Get the cf_clearance token
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Get the cf_clearance cookie value from the cookies map
    pub fn cf_clearance(&self) -> Option<&str> {
        self.cookies
            .as_ref()
            .and_then(|c| c.get("cf_clearance").map(|s| s.as_str()))
    }
}

// ============================================================================
// Internal Types (Request/Response)
// ============================================================================

/// Response data from Capsolver createTask endpoint (success case)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateTaskData {
    pub task_id: String,
}

/// Response data from Capsolver getTaskResult endpoint (success case)
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
    pub(crate) task: &'a CapsolverTask,
}

/// Request payload for getting task result
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetTaskResultRequest<'a> {
    pub(crate) client_key: &'a str,
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
        let task = CapsolverTask::recaptcha_v2("https://example.com", "site-key");
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("ReCaptchaV2TaskProxyLess"));
        assert!(json.contains("websiteURL"));
        assert!(json.contains("websiteKey"));
    }

    #[test]
    fn test_recaptcha_v2_invisible_serialization() {
        let task = CapsolverTask::recaptcha_v2_invisible("https://example.com", "site-key");
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("isInvisible"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_recaptcha_v3_with_action_serialization() {
        let task =
            CapsolverTask::recaptcha_v3_with_action("https://example.com", "site-key", "submit");
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("ReCaptchaV3TaskProxyLess"));
        assert!(json.contains("pageAction"));
        assert!(json.contains("submit"));
    }

    #[test]
    fn test_turnstile_serialization() {
        let task = CapsolverTask::turnstile("https://example.com", "site-key");
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("AntiTurnstileTaskProxyLess"));
        assert!(!json.contains("metadata"));
    }

    #[test]
    fn test_turnstile_with_metadata_serialization() {
        let metadata = TurnstileMetadata {
            action: Some("login".to_string()),
            cdata: None,
        };
        let task =
            CapsolverTask::turnstile_with_metadata("https://example.com", "site-key", metadata);
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("metadata"));
        assert!(json.contains("action"));
        assert!(json.contains("login"));
    }

    #[test]
    fn test_cloudflare_challenge_serialization() {
        let task = CapsolverTask::cloudflare_challenge("https://example.com", "http://proxy:8080");
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("AntiCloudflareTask"));
        assert!(json.contains("proxy"));
    }

    #[test]
    fn test_recaptcha_solution_deserialization() {
        let json = r#"{
            "gRecaptchaResponse": "token-value",
            "userAgent": "Mozilla/5.0",
            "secChUa": "Chromium",
            "createTime": 1234567890,
            "recaptcha-ca-t": "session-cookie",
            "recaptcha-ca-e": "v2-cookie"
        }"#;
        let solution: ReCaptchaSolution = serde_json::from_str(json).unwrap();
        assert_eq!(solution.token(), "token-value");
        assert_eq!(solution.user_agent.as_deref(), Some("Mozilla/5.0"));
        assert_eq!(solution.sec_ch_ua.as_deref(), Some("Chromium"));
        assert_eq!(solution.create_time, Some(1234567890));
        assert_eq!(solution.session_cookie(), Some("session-cookie"));
        assert_eq!(solution.recaptcha_ca_e.as_deref(), Some("v2-cookie"));
    }

    #[test]
    fn test_turnstile_solution_deserialization() {
        let json = r#"{
            "token": "turnstile-token",
            "userAgent": "Mozilla/5.0"
        }"#;
        let solution: TurnstileSolution = serde_json::from_str(json).unwrap();
        assert_eq!(solution.token(), "turnstile-token");
    }

    #[test]
    fn test_cloudflare_solution_deserialization() {
        let json = r#"{
            "token": "cf-token",
            "cookies": {"cf_clearance": "clearance-value"},
            "userAgent": "Mozilla/5.0"
        }"#;
        let solution: CloudflareChallengeSolution = serde_json::from_str(json).unwrap();
        assert_eq!(solution.token(), "cf-token");
        assert_eq!(solution.cf_clearance(), Some("clearance-value"));
    }

    #[test]
    fn test_task_display() {
        assert_eq!(
            CapsolverTask::recaptcha_v2("url", "key").to_string(),
            "ReCaptchaV2"
        );
        assert_eq!(
            CapsolverTask::recaptcha_v3("url", "key").to_string(),
            "ReCaptchaV3"
        );
        assert_eq!(
            CapsolverTask::turnstile("url", "key").to_string(),
            "Turnstile"
        );
        assert_eq!(
            CapsolverTask::cloudflare_challenge("url", "proxy").to_string(),
            "CloudflareChallenge"
        );
    }
}
use crate::utils::proxy::CapsolverProxyFields;
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
        #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
        page_action: Option<String>,
        #[serde(rename = "enterprisePayload", skip_serializing_if = "Option::is_none")]
        enterprise_payload: Option<HashMap<String, serde_json::Value>>,
        #[serde(rename = "isInvisible", skip_serializing_if = "Option::is_none")]
        is_invisible: Option<bool>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
        #[serde(flatten)]
        proxy: CapsolverProxyFields,
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
        #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
        page_action: Option<String>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
        #[serde(flatten)]
        proxy: CapsolverProxyFields,
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
        #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
        page_action: Option<String>,
        #[serde(rename = "enterprisePayload", skip_serializing_if = "Option::is_none")]
        enterprise_payload: Option<HashMap<String, serde_json::Value>>,
        #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
        api_domain: Option<String>,
        #[serde(flatten)]
        proxy: CapsolverProxyFields,
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
        #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
        user_agent: Option<String>,
        /// Response HTML from target website (typically contains "Just a moment...")
        #[serde(skip_serializing_if = "Option::is_none")]
        html: Option<String>,
        /// Proxy fields (static or sticky proxy, not rotating)
        #[serde(flatten)]
        proxy: CapsolverProxyFields,
    },

    // -------------------------------------------------------------------------
    // Image to Text
    // -------------------------------------------------------------------------
    /// Image to text OCR recognition
    ImageToTextTask {
        /// Base64 encoded image content (no newlines, no data:image prefix)
        body: String,
        /// Page source URL to improve accuracy
        #[serde(rename = "websiteURL", skip_serializing_if = "Option::is_none")]
        website_url: Option<String>,
        /// Recognition module (e.g., "common", "number")
        #[serde(skip_serializing_if = "Option::is_none")]
        module: Option<String>,
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
            Self::ImageToTextTask { .. } => write!(f, "ImageToText"),
        }
    }
}

// ============================================================================
// Solution Types
// ============================================================================

// Re-export shared solution types for convenience
pub use crate::solutions::{
    CloudflareChallengeSolution, ImageToTextSolution, ReCaptchaSolution, TurnstileSolution,
};

/// Capsolver solution types
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CapsolverSolution {
    /// Image to text solution (must be first for untagged deserialization priority)
    ImageToText(ImageToTextSolution),
    /// ReCaptcha solution (V2 or V3)
    ReCaptcha(ReCaptchaSolution),
    /// Turnstile or Cloudflare Challenge solution
    Turnstile(TurnstileSolution),
}

impl crate::solutions::ProviderSolution for CapsolverSolution {}

impl CapsolverSolution {
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
    pub fn try_into_recaptcha(self) -> Result<ReCaptchaSolution, Box<Self>> {
        match self {
            Self::ReCaptcha(solution) => Ok(solution),
            other => Err(Box::new(other)),
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

    /// Try to extract Turnstile/Cloudflare Challenge solution (returns reference)
    pub fn as_turnstile(&self) -> Option<&TurnstileSolution> {
        match self {
            Self::Turnstile(solution) => Some(solution),
            _ => None,
        }
    }

    /// Try to extract Turnstile solution (consumes self)
    ///
    /// Returns `Ok(solution)` if this is a Turnstile solution, or `Err(self)` otherwise.
    pub fn try_into_turnstile(self) -> Result<TurnstileSolution, Box<Self>> {
        match self {
            Self::Turnstile(solution) => Ok(solution),
            other => Err(Box::new(other)),
        }
    }

    /// Extract Turnstile/Cloudflare Challenge solution, panics if not Turnstile
    ///
    /// # Panics
    /// Panics if the solution is not a Turnstile solution.
    /// Use `try_into_turnstile()` for a non-panicking alternative.
    pub fn into_turnstile(self) -> TurnstileSolution {
        self.try_into_turnstile()
            .expect("Expected Turnstile solution")
    }

    /// Alias for `as_turnstile` - Cloudflare Challenge uses the same solution type
    pub fn as_cloudflare_challenge(&self) -> Option<&TurnstileSolution> {
        self.as_turnstile()
    }

    /// Alias for `try_into_turnstile` - Cloudflare Challenge uses the same solution type
    pub fn try_into_cloudflare_challenge(self) -> Result<TurnstileSolution, Box<Self>> {
        self.try_into_turnstile()
    }

    /// Alias for `into_turnstile` - Cloudflare Challenge uses the same solution type
    pub fn into_cloudflare_challenge(self) -> TurnstileSolution {
        self.into_turnstile()
    }

    /// Try to extract ImageToText solution (returns reference)
    pub fn as_image_to_text(&self) -> Option<&ImageToTextSolution> {
        match self {
            Self::ImageToText(solution) => Some(solution),
            _ => None,
        }
    }

    /// Try to extract ImageToText solution (consumes self)
    ///
    /// Returns `Ok(solution)` if this is an ImageToText solution, or `Err(self)` otherwise.
    pub fn try_into_image_to_text(self) -> Result<ImageToTextSolution, Box<Self>> {
        match self {
            Self::ImageToText(solution) => Ok(solution),
            other => Err(Box::new(other)),
        }
    }

    /// Extract ImageToText solution, panics if not ImageToText
    ///
    /// # Panics
    /// Panics if the solution is not an ImageToText solution.
    /// Use `try_into_image_to_text()` for a non-panicking alternative.
    pub fn into_image_to_text(self) -> ImageToTextSolution {
        self.try_into_image_to_text()
            .expect("Expected ImageToText solution")
    }
}

// ============================================================================
// Internal Types (Request/Response)
// ============================================================================

/// Response data from Capsolver createTask endpoint (success case)
///
/// Some task types (like ImageToText) return the solution immediately
/// in the createTask response without requiring a separate getTaskResult call.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateTaskData {
    pub task_id: String,
    /// Solution returned immediately (for synchronous tasks like ImageToText)
    #[serde(default)]
    pub solution: Option<CapsolverSolution>,
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
// From implementations for shared task types
// ============================================================================

impl TryFrom<crate::tasks::ReCaptchaV2> for CapsolverTask {
    type Error = crate::errors::UnsupportedTaskError;

    fn try_from(task: crate::tasks::ReCaptchaV2) -> Result<Self, Self::Error> {
        let is_invisible = if task.is_invisible { Some(true) } else { None };

        match (task.is_enterprise, task.proxy) {
            // Enterprise with proxy
            (true, Some(proxy)) => Ok(Self::ReCaptchaV2EnterpriseTask {
                website_url: task.website_url,
                website_key: task.website_key,
                page_action: task.page_action,
                enterprise_payload: task.enterprise_payload,
                is_invisible,
                api_domain: task.api_domain,
                proxy: proxy.into_capsolver_fields(),
            }),
            // Enterprise without proxy
            (true, None) => Ok(Self::ReCaptchaV2EnterpriseTaskProxyLess {
                website_url: task.website_url,
                website_key: task.website_key,
                page_action: task.page_action,
                enterprise_payload: task.enterprise_payload,
                is_invisible,
                api_domain: task.api_domain,
            }),
            // Standard with proxy — Capsolver does not support V2 non-enterprise with proxy
            (false, Some(_)) => Err(crate::errors::UnsupportedTaskError::unsupported_fields(
                "ReCaptchaV2",
                "Capsolver",
                vec!["proxy (non-enterprise)"],
            )),
            // Standard without proxy
            (false, None) => Ok(Self::ReCaptchaV2TaskProxyLess {
                website_url: task.website_url,
                website_key: task.website_key,
                page_action: task.page_action,
                recaptcha_data_s_value: task.recaptcha_data_s_value,
                is_invisible,
                api_domain: task.api_domain,
            }),
        }
    }
}

impl From<crate::tasks::ReCaptchaV3> for CapsolverTask {
    fn from(task: crate::tasks::ReCaptchaV3) -> Self {
        match (task.is_enterprise, task.proxy) {
            // Enterprise with proxy
            (true, Some(proxy)) => Self::ReCaptchaV3EnterpriseTask {
                website_url: task.website_url,
                website_key: task.website_key,
                page_action: task.page_action,
                enterprise_payload: task.enterprise_payload,
                api_domain: task.api_domain,
                proxy: proxy.into_capsolver_fields(),
            },
            // Enterprise without proxy
            (true, None) => Self::ReCaptchaV3EnterpriseTaskProxyLess {
                website_url: task.website_url,
                website_key: task.website_key,
                page_action: task.page_action,
                enterprise_payload: task.enterprise_payload,
                api_domain: task.api_domain,
            },
            // Standard with proxy
            (false, Some(proxy)) => Self::ReCaptchaV3Task {
                website_url: task.website_url,
                website_key: task.website_key,
                page_action: task.page_action,
                api_domain: task.api_domain,
                proxy: proxy.into_capsolver_fields(),
            },
            // Standard without proxy
            (false, None) => Self::ReCaptchaV3TaskProxyLess {
                website_url: task.website_url,
                website_key: task.website_key,
                page_action: task.page_action,
                api_domain: task.api_domain,
            },
        }
    }
}

impl TryFrom<crate::tasks::Turnstile> for CapsolverTask {
    type Error = crate::errors::UnsupportedTaskError;

    fn try_from(task: crate::tasks::Turnstile) -> Result<Self, Self::Error> {
        let mut unsupported = Vec::new();
        if task.proxy.is_some() {
            unsupported.push("proxy");
        }
        if task.pagedata.is_some() {
            unsupported.push("pagedata");
        }
        if !unsupported.is_empty() {
            return Err(crate::errors::UnsupportedTaskError::unsupported_fields(
                "Turnstile",
                "Capsolver",
                unsupported,
            ));
        }

        let metadata = if task.action.is_some() || task.cdata.is_some() {
            Some(TurnstileMetadata {
                action: task.action,
                cdata: task.cdata,
            })
        } else {
            None
        };

        Ok(Self::AntiTurnstileTaskProxyLess {
            website_url: task.website_url,
            website_key: task.website_key,
            metadata,
        })
    }
}

impl From<crate::tasks::CloudflareChallenge> for CapsolverTask {
    fn from(task: crate::tasks::CloudflareChallenge) -> Self {
        Self::AntiCloudflareTask {
            website_url: task.website_url,
            user_agent: task.user_agent,
            html: task.html,
            proxy: task.proxy.into_capsolver_fields(),
        }
    }
}

impl TryFrom<crate::tasks::ImageToText> for CapsolverTask {
    type Error = crate::errors::UnsupportedTaskError;

    fn try_from(task: crate::tasks::ImageToText) -> Result<Self, Self::Error> {
        let mut unsupported = Vec::new();
        if task.phrase {
            unsupported.push("phrase");
        }
        if task.case_sensitive {
            unsupported.push("case_sensitive");
        }
        if task.numeric != 0 {
            unsupported.push("numeric");
        }
        if task.math {
            unsupported.push("math");
        }
        if task.min_length > 0 {
            unsupported.push("min_length");
        }
        if task.max_length > 0 {
            unsupported.push("max_length");
        }
        if task.comment.is_some() {
            unsupported.push("comment");
        }
        if task.img_instructions.is_some() {
            unsupported.push("img_instructions");
        }
        if !unsupported.is_empty() {
            return Err(crate::errors::UnsupportedTaskError::unsupported_fields(
                "ImageToText",
                "Capsolver",
                unsupported,
            ));
        }

        Ok(Self::ImageToTextTask {
            body: task.body,
            website_url: task.website_url,
            module: task.module,
        })
    }
}

impl TryFrom<crate::tasks::CaptchaTask> for CapsolverTask {
    type Error = crate::errors::UnsupportedTaskError;

    fn try_from(task: crate::tasks::CaptchaTask) -> Result<Self, Self::Error> {
        match task {
            crate::tasks::CaptchaTask::ReCaptchaV2(t) => t.try_into(),
            crate::tasks::CaptchaTask::ReCaptchaV3(t) => Ok(t.into()),
            crate::tasks::CaptchaTask::Turnstile(t) => t.try_into(),
            crate::tasks::CaptchaTask::CloudflareChallenge(t) => Ok(t.into()),
            crate::tasks::CaptchaTask::ImageToText(t) => t.try_into(),
            crate::tasks::CaptchaTask::TurnstileChallenge(_) => Err(
                crate::errors::UnsupportedTaskError::new("TurnstileChallenge", "Capsolver"),
            ),
            crate::tasks::CaptchaTask::TurnstileWaitRoom(_) => Err(
                crate::errors::UnsupportedTaskError::new("TurnstileWaitRoom", "Capsolver"),
            ),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::{CloudflareChallenge, ReCaptchaV2, ReCaptchaV3, Turnstile};
    use crate::utils::proxy::ProxyConfig;

    #[test]
    fn test_recaptcha_v2_serialization() {
        let task: CapsolverTask = ReCaptchaV2::new("https://example.com", "site-key")
            .try_into()
            .unwrap();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("ReCaptchaV2TaskProxyLess"));
        assert!(json.contains("websiteURL"));
        assert!(json.contains("websiteKey"));
    }

    #[test]
    fn test_recaptcha_v2_invisible_serialization() {
        let task: CapsolverTask = ReCaptchaV2::new("https://example.com", "site-key")
            .invisible()
            .try_into()
            .unwrap();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("isInvisible"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_recaptcha_v3_with_action_serialization() {
        let task: CapsolverTask = ReCaptchaV3::new("https://example.com", "site-key")
            .with_action("submit")
            .into();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("ReCaptchaV3TaskProxyLess"));
        assert!(json.contains("pageAction"));
        assert!(json.contains("submit"));
    }

    #[test]
    fn test_turnstile_serialization() {
        let task: CapsolverTask = Turnstile::new("https://example.com", "site-key")
            .try_into()
            .unwrap();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("AntiTurnstileTaskProxyLess"));
    }

    #[test]
    fn test_turnstile_with_metadata_serialization() {
        let task: CapsolverTask = Turnstile::new("https://example.com", "site-key")
            .with_action("login")
            .try_into()
            .unwrap();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("metadata"));
        assert!(json.contains("action"));
        assert!(json.contains("login"));
    }

    #[test]
    fn test_cloudflare_challenge_serialization() {
        let proxy = ProxyConfig::http("proxy.example.com", 8080);
        let task: CapsolverTask = CloudflareChallenge::new("https://example.com", proxy).into();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("AntiCloudflareTask"));
        assert!(json.contains("proxyType"));
        assert!(json.contains("proxyAddress"));
        assert!(json.contains("proxyPort"));
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
        assert_eq!(solution.token().unwrap(), "turnstile-token");
    }

    #[test]
    fn test_cloudflare_solution_deserialization() {
        let json = r#"{
            "token": "cf-token",
            "cookies": {"cf_clearance": "clearance-value"},
            "userAgent": "Mozilla/5.0"
        }"#;
        let solution: CloudflareChallengeSolution = serde_json::from_str(json).unwrap();
        assert_eq!(solution.token().unwrap(), "cf-token");
        assert_eq!(solution.cf_clearance(), Some("clearance-value"));
    }

    #[test]
    fn test_task_display() {
        let task: CapsolverTask = ReCaptchaV2::new("url", "key").try_into().unwrap();
        assert_eq!(task.to_string(), "ReCaptchaV2");

        let task: CapsolverTask = ReCaptchaV3::new("url", "key").into();
        assert_eq!(task.to_string(), "ReCaptchaV3");

        let task: CapsolverTask = Turnstile::new("url", "key").try_into().unwrap();
        assert_eq!(task.to_string(), "Turnstile");

        let proxy = ProxyConfig::http("proxy", 8080);
        let task: CapsolverTask = CloudflareChallenge::new("url", proxy).into();
        assert_eq!(task.to_string(), "CloudflareChallenge");
    }

    #[test]
    fn test_proxy_serialization() {
        let proxy = ProxyConfig::socks5("192.168.1.1", 1080).with_auth("user", "pass");
        let task: CapsolverTask = ReCaptchaV3::new("https://example.com", "key")
            .with_proxy(proxy)
            .into();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("\"proxyType\":\"socks5\""));
        assert!(json.contains("\"proxyAddress\":\"192.168.1.1\""));
        assert!(json.contains("\"proxyPort\":1080"));
        assert!(json.contains("\"proxyLogin\":\"user\""));
        assert!(json.contains("\"proxyPassword\":\"pass\""));
    }

    #[test]
    fn test_from_shared_recaptcha_v2() {
        let task = ReCaptchaV2::new("https://example.com", "key");
        let capsolver_task: CapsolverTask = task.try_into().unwrap();
        let json = serde_json::to_string(&capsolver_task).unwrap();
        assert!(json.contains("ReCaptchaV2TaskProxyLess"));
    }

    #[test]
    fn test_from_shared_recaptcha_v2_enterprise_with_proxy() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        let task = ReCaptchaV2::new("https://example.com", "key")
            .enterprise()
            .with_proxy(proxy);
        let capsolver_task: CapsolverTask = task.try_into().unwrap();
        let json = serde_json::to_string(&capsolver_task).unwrap();
        assert!(json.contains("ReCaptchaV2EnterpriseTask"));
        assert!(json.contains("proxyAddress"));
    }

    #[test]
    fn test_from_shared_turnstile() {
        let task = Turnstile::new("https://example.com", "key").with_action("login");
        let capsolver_task: CapsolverTask = task.try_into().unwrap();
        let json = serde_json::to_string(&capsolver_task).unwrap();
        assert!(json.contains("AntiTurnstileTaskProxyLess"));
        assert!(json.contains("\"action\":\"login\""));
    }

    #[test]
    fn test_image_to_text_serialization() {
        use crate::tasks::ImageToText;
        let task: CapsolverTask = ImageToText::from_base64("aVZCT1J3MEtHZ29B")
            .try_into()
            .unwrap();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("ImageToTextTask"));
        assert!(json.contains("\"body\":\"aVZCT1J3MEtHZ29B\""));
    }

    #[test]
    fn test_image_to_text_with_module_serialization() {
        use crate::tasks::ImageToText;
        let task: CapsolverTask = ImageToText::from_base64("base64data")
            .with_module("number")
            .with_website_url("https://example.com")
            .try_into()
            .unwrap();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("ImageToTextTask"));
        assert!(json.contains("\"module\":\"number\""));
        assert!(json.contains("\"websiteURL\":\"https://example.com\""));
    }

    #[test]
    fn test_image_to_text_solution_deserialization() {
        let json = r#"{"text": "ABC123"}"#;
        let solution: ImageToTextSolution = serde_json::from_str(json).unwrap();
        assert_eq!(solution.text(), "ABC123");
    }

    #[test]
    fn test_image_to_text_display() {
        use crate::tasks::ImageToText;
        let task: CapsolverTask = ImageToText::from_base64("data").try_into().unwrap();
        assert_eq!(task.to_string(), "ImageToText");
    }

    // === Rejection tests for unsupported field combinations ===

    #[test]
    fn test_recaptcha_v2_non_enterprise_with_proxy_rejected() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        let task = ReCaptchaV2::new("https://example.com", "key").with_proxy(proxy);
        let result: Result<CapsolverTask, _> = task.try_into();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.task_type, "ReCaptchaV2");
        assert!(!err.unsupported_fields.is_empty());
    }

    #[test]
    fn test_turnstile_with_proxy_rejected() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        let task = Turnstile::new("https://example.com", "key").with_proxy(proxy);
        let result: Result<CapsolverTask, _> = task.try_into();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.unsupported_fields.contains(&"proxy"));
    }

    #[test]
    fn test_turnstile_with_pagedata_rejected() {
        let task = Turnstile::new("https://example.com", "key").with_pagedata("data");
        let result: Result<CapsolverTask, _> = task.try_into();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.unsupported_fields.contains(&"pagedata"));
    }

    #[test]
    fn test_image_to_text_with_ocr_fields_rejected() {
        use crate::tasks::ImageToText;
        let task = ImageToText::from_base64("data")
            .case_sensitive()
            .numbers_only();
        let result: Result<CapsolverTask, _> = task.try_into();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.unsupported_fields.contains(&"case_sensitive"));
        assert!(err.unsupported_fields.contains(&"numeric"));
    }
}

//! Task and solution types for the CapMonster Cloud API.

use crate::errors::UnsupportedTaskError;
use crate::utils::proxy::ApiProxyFields;
use crate::utils::serde_helpers::{
    deserialize_string_or_number, serialize_string_as_number_if_possible,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

// ============================================================================
// Task Types
// ============================================================================

/// CapMonster task types for API request payloads.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum CapmonsterTask {
    // -------------------------------------------------------------------------
    // ReCaptcha V2
    // -------------------------------------------------------------------------
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
        #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
        user_agent: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cookies: Option<String>,
        #[serde(rename = "isInvisible", skip_serializing_if = "Option::is_none")]
        is_invisible: Option<bool>,
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        proxy: Option<ApiProxyFields>,
    },

    // -------------------------------------------------------------------------
    // ReCaptcha V2 Enterprise
    // -------------------------------------------------------------------------
    RecaptchaV2EnterpriseTask {
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
        #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
        user_agent: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cookies: Option<String>,
        #[serde(rename = "isInvisible", skip_serializing_if = "Option::is_none")]
        is_invisible: Option<bool>,
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        proxy: Option<ApiProxyFields>,
    },

    // -------------------------------------------------------------------------
    // ReCaptcha V3
    // -------------------------------------------------------------------------
    RecaptchaV3TaskProxyless {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(rename = "minScore", skip_serializing_if = "Option::is_none")]
        min_score: Option<f32>,
        #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
        page_action: Option<String>,
        #[serde(rename = "isEnterprise", skip_serializing_if = "Option::is_none")]
        is_enterprise: Option<bool>,
    },

    RecaptchaV3EnterpriseTask {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(rename = "minScore", skip_serializing_if = "Option::is_none")]
        min_score: Option<f32>,
        #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
        page_action: Option<String>,
    },

    // -------------------------------------------------------------------------
    // Turnstile / Challenge / Wait Room
    // -------------------------------------------------------------------------
    TurnstileTask {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
        page_action: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(rename = "pageData", skip_serializing_if = "Option::is_none")]
        page_data: Option<String>,
        #[serde(rename = "cloudflareTaskType", skip_serializing_if = "Option::is_none")]
        cloudflare_task_type: Option<String>,
        #[serde(rename = "apiJsUrl", skip_serializing_if = "Option::is_none")]
        api_js_url: Option<String>,
        #[serde(rename = "htmlPageBase64", skip_serializing_if = "Option::is_none")]
        html_page_base64: Option<String>,
        #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
        user_agent: Option<String>,
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        proxy: Option<ApiProxyFields>,
    },

    // -------------------------------------------------------------------------
    // Image to Text
    // -------------------------------------------------------------------------
    ImageToTextTask {
        body: String,
        #[serde(rename = "CapMonsterModule", skip_serializing_if = "Option::is_none")]
        module: Option<String>,
    },
}

impl Display for CapmonsterTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecaptchaV2Task { .. } => write!(f, "ReCaptchaV2"),
            Self::RecaptchaV2EnterpriseTask { .. } => write!(f, "ReCaptchaV2Enterprise"),
            Self::RecaptchaV3TaskProxyless {
                is_enterprise: Some(true),
                ..
            }
            | Self::RecaptchaV3EnterpriseTask { .. } => write!(f, "ReCaptchaV3Enterprise"),
            Self::RecaptchaV3TaskProxyless { .. } => write!(f, "ReCaptchaV3"),
            Self::TurnstileTask {
                cloudflare_task_type: Some(task_type),
                ..
            } => match task_type.as_str() {
                "token" => write!(f, "TurnstileChallenge"),
                "cf_clearance" => write!(f, "TurnstileChallengeCfClearance"),
                "wait_room" => write!(f, "TurnstileWaitRoom"),
                _ => write!(f, "Turnstile"),
            },
            Self::TurnstileTask { .. } => write!(f, "Turnstile"),
            Self::ImageToTextTask { .. } => write!(f, "ImageToText"),
        }
    }
}

// ============================================================================
// Solution Types
// ============================================================================

// Re-export shared solution types for convenience.
pub use crate::solutions::{ImageToTextSolution, ReCaptchaSolution, TurnstileSolution};

/// CapMonster solution types.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CapmonsterSolution {
    /// Image to text solution (must be first for untagged deserialization priority —
    /// `ImageToTextSolution` has a unique `text` field)
    ImageToText(ImageToTextSolution),
    ReCaptcha(ReCaptchaSolution),
    Turnstile(TurnstileSolution),
}

impl crate::solutions::ProviderSolution for CapmonsterSolution {}

impl CapmonsterSolution {
    /// Try to extract ReCaptcha solution (returns reference).
    pub fn as_recaptcha(&self) -> Option<&ReCaptchaSolution> {
        match self {
            Self::ReCaptcha(solution) => Some(solution),
            _ => None,
        }
    }

    /// Try to extract ReCaptcha solution (consumes self).
    pub fn try_into_recaptcha(self) -> Result<ReCaptchaSolution, Box<Self>> {
        match self {
            Self::ReCaptcha(solution) => Ok(solution),
            other => Err(Box::new(other)),
        }
    }

    /// Extract ReCaptcha solution, panics if not ReCaptcha.
    pub fn into_recaptcha(self) -> ReCaptchaSolution {
        self.try_into_recaptcha()
            .expect("Expected ReCaptcha solution")
    }

    /// Try to extract Turnstile solution (returns reference).
    pub fn as_turnstile(&self) -> Option<&TurnstileSolution> {
        match self {
            Self::Turnstile(solution) => Some(solution),
            _ => None,
        }
    }

    /// Try to extract Turnstile solution (consumes self).
    pub fn try_into_turnstile(self) -> Result<TurnstileSolution, Box<Self>> {
        match self {
            Self::Turnstile(solution) => Ok(solution),
            other => Err(Box::new(other)),
        }
    }

    /// Extract Turnstile solution, panics if not Turnstile.
    pub fn into_turnstile(self) -> TurnstileSolution {
        self.try_into_turnstile()
            .expect("Expected Turnstile solution")
    }

    /// Try to extract ImageToText solution (returns reference).
    pub fn as_image_to_text(&self) -> Option<&ImageToTextSolution> {
        match self {
            Self::ImageToText(solution) => Some(solution),
            _ => None,
        }
    }

    /// Try to extract ImageToText solution (consumes self).
    pub fn try_into_image_to_text(self) -> Result<ImageToTextSolution, Box<Self>> {
        match self {
            Self::ImageToText(solution) => Ok(solution),
            other => Err(Box::new(other)),
        }
    }

    /// Extract ImageToText solution, panics if not ImageToText.
    pub fn into_image_to_text(self) -> ImageToTextSolution {
        self.try_into_image_to_text()
            .expect("Expected ImageToText solution")
    }
}

// ============================================================================
// Internal Types (Request/Response)
// ============================================================================

/// Response data from createTask endpoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateTaskData {
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub task_id: String,
}

/// Response data from getTaskResult endpoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetTaskData<T> {
    #[allow(dead_code)]
    pub status: String,
    pub solution: Option<T>,
}

/// Request payload for creating a task.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateTaskRequest<'a> {
    pub(crate) client_key: &'a str,
    pub(crate) task: &'a CapmonsterTask,
}

/// Request payload for getting task result.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetTaskResultRequest<'a> {
    pub(crate) client_key: &'a str,
    #[serde(serialize_with = "serialize_string_as_number_if_possible")]
    pub(crate) task_id: &'a str,
}

// ============================================================================
// From/TryFrom implementations for shared task types
// ============================================================================

impl TryFrom<crate::tasks::ReCaptchaV2> for CapmonsterTask {
    type Error = UnsupportedTaskError;

    fn try_from(task: crate::tasks::ReCaptchaV2) -> Result<Self, Self::Error> {
        let is_invisible = if task.is_invisible { Some(true) } else { None };
        let proxy = task.proxy.map(|p| p.into_api_proxy_fields());

        if task.is_enterprise {
            let mut unsupported = Vec::new();
            if task.recaptcha_data_s_value.is_some() {
                unsupported.push("recaptcha_data_s_value");
            }
            if !unsupported.is_empty() {
                return Err(UnsupportedTaskError::unsupported_fields(
                    "ReCaptchaV2",
                    "CapMonster",
                    unsupported,
                ));
            }

            Ok(Self::RecaptchaV2EnterpriseTask {
                website_url: task.website_url,
                website_key: task.website_key,
                page_action: task.page_action,
                enterprise_payload: task.enterprise_payload,
                api_domain: task.api_domain,
                user_agent: task.user_agent,
                cookies: task.cookies,
                is_invisible,
                proxy,
            })
        } else {
            let mut unsupported = Vec::new();
            if task.page_action.is_some() {
                unsupported.push("page_action");
            }
            if task.api_domain.is_some() {
                unsupported.push("api_domain");
            }
            if task.enterprise_payload.is_some() {
                unsupported.push("enterprise_payload");
            }
            if !unsupported.is_empty() {
                return Err(UnsupportedTaskError::unsupported_fields(
                    "ReCaptchaV2",
                    "CapMonster",
                    unsupported,
                ));
            }

            Ok(Self::RecaptchaV2Task {
                website_url: task.website_url,
                website_key: task.website_key,
                recaptcha_data_s_value: task.recaptcha_data_s_value,
                user_agent: task.user_agent,
                cookies: task.cookies,
                is_invisible,
                proxy,
            })
        }
    }
}

impl TryFrom<crate::tasks::ReCaptchaV3> for CapmonsterTask {
    type Error = UnsupportedTaskError;

    fn try_from(task: crate::tasks::ReCaptchaV3) -> Result<Self, Self::Error> {
        let mut unsupported = Vec::new();
        if task.proxy.is_some() {
            unsupported.push("proxy");
        }
        if task.api_domain.is_some() {
            unsupported.push("api_domain");
        }
        if task.enterprise_payload.is_some() {
            unsupported.push("enterprise_payload");
        }
        if !unsupported.is_empty() {
            return Err(UnsupportedTaskError::unsupported_fields(
                "ReCaptchaV3",
                "CapMonster",
                unsupported,
            ));
        }

        if task.is_enterprise {
            Ok(Self::RecaptchaV3EnterpriseTask {
                website_url: task.website_url,
                website_key: task.website_key,
                min_score: task.min_score,
                page_action: task.page_action,
            })
        } else {
            Ok(Self::RecaptchaV3TaskProxyless {
                website_url: task.website_url,
                website_key: task.website_key,
                min_score: task.min_score,
                page_action: task.page_action,
                is_enterprise: None,
            })
        }
    }
}

impl From<crate::tasks::Turnstile> for CapmonsterTask {
    fn from(task: crate::tasks::Turnstile) -> Self {
        Self::TurnstileTask {
            website_url: task.website_url,
            website_key: task.website_key,
            page_action: task.action,
            data: task.cdata,
            page_data: task.pagedata,
            cloudflare_task_type: None,
            api_js_url: None,
            html_page_base64: None,
            user_agent: None,
            proxy: task.proxy.map(|p| p.into_api_proxy_fields()),
        }
    }
}

impl From<crate::tasks::TurnstileChallenge> for CapmonsterTask {
    fn from(task: crate::tasks::TurnstileChallenge) -> Self {
        let cloudflare_task_type = Some(task.mode.as_cloudflare_task_type().to_string());

        Self::TurnstileTask {
            website_url: task.website_url,
            website_key: task.website_key,
            page_action: task.page_action,
            data: task.data,
            page_data: task.page_data,
            cloudflare_task_type,
            api_js_url: task.api_js_url,
            html_page_base64: task.html_page_base64,
            user_agent: Some(task.user_agent),
            proxy: task.proxy.map(|p| p.into_api_proxy_fields()),
        }
    }
}

impl From<crate::tasks::TurnstileWaitRoom> for CapmonsterTask {
    fn from(task: crate::tasks::TurnstileWaitRoom) -> Self {
        Self::TurnstileTask {
            website_url: task.website_url,
            website_key: task.website_key,
            page_action: None,
            data: None,
            page_data: None,
            cloudflare_task_type: Some("wait_room".to_string()),
            api_js_url: None,
            html_page_base64: Some(task.html_page_base64),
            user_agent: Some(task.user_agent),
            proxy: Some(task.proxy.into_api_proxy_fields()),
        }
    }
}

impl TryFrom<crate::tasks::ImageToText> for CapmonsterTask {
    type Error = UnsupportedTaskError;

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
            return Err(UnsupportedTaskError::unsupported_fields(
                "ImageToText",
                "CapMonster",
                unsupported,
            ));
        }

        Ok(Self::ImageToTextTask {
            body: task.body,
            module: task.module,
        })
    }
}

impl TryFrom<crate::tasks::CaptchaTask> for CapmonsterTask {
    type Error = UnsupportedTaskError;

    fn try_from(task: crate::tasks::CaptchaTask) -> Result<Self, Self::Error> {
        match task {
            crate::tasks::CaptchaTask::ReCaptchaV2(t) => t.try_into(),
            crate::tasks::CaptchaTask::ReCaptchaV3(t) => t.try_into(),
            crate::tasks::CaptchaTask::Turnstile(t) => Ok(t.into()),
            crate::tasks::CaptchaTask::TurnstileChallenge(t) => Ok(t.into()),
            crate::tasks::CaptchaTask::TurnstileWaitRoom(t) => Ok(t.into()),
            crate::tasks::CaptchaTask::CloudflareChallenge(_) => Err(UnsupportedTaskError::new(
                "CloudflareChallenge",
                "CapMonster",
            )),
            crate::tasks::CaptchaTask::ImageToText(t) => t.try_into(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::{
        ImageToText, ReCaptchaV2, ReCaptchaV3, Turnstile, TurnstileChallenge, TurnstileWaitRoom,
    };
    use crate::utils::proxy::ProxyConfig;

    #[test]
    fn test_recaptcha_v2_serialization() {
        let task: CapmonsterTask = ReCaptchaV2::new("https://example.com", "site-key")
            .try_into()
            .unwrap();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("RecaptchaV2Task"));
        assert!(json.contains("websiteURL"));
        assert!(json.contains("websiteKey"));
    }

    #[test]
    fn test_recaptcha_v2_enterprise_serialization() {
        let task: CapmonsterTask = ReCaptchaV2::new("https://example.com", "site-key")
            .enterprise()
            .try_into()
            .unwrap();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("RecaptchaV2EnterpriseTask"));
    }

    #[test]
    fn test_recaptcha_v2_rejects_enterprise_fields_on_non_enterprise() {
        let task = ReCaptchaV2::new("https://example.com", "key")
            .with_action("verify")
            .with_api_domain("recaptcha.net");
        let result: Result<CapmonsterTask, _> = task.try_into();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.unsupported_fields.contains(&"page_action"));
        assert!(err.unsupported_fields.contains(&"api_domain"));
    }

    #[test]
    fn test_recaptcha_v2_rejects_data_s_on_enterprise() {
        let task = ReCaptchaV2::new("https://example.com", "key")
            .enterprise()
            .with_data_s_value("token");
        let result: Result<CapmonsterTask, _> = task.try_into();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.unsupported_fields.contains(&"recaptcha_data_s_value"));
    }

    #[test]
    fn test_recaptcha_v3_enterprise_serialization() {
        let task: CapmonsterTask = ReCaptchaV3::new("https://example.com", "site-key")
            .enterprise()
            .with_min_score(0.7)
            .try_into()
            .unwrap();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("RecaptchaV3EnterpriseTask"));
        assert!(json.contains("minScore"));
    }

    #[test]
    fn test_turnstile_serialization() {
        let task: CapmonsterTask = Turnstile::new("https://example.com", "site-key")
            .with_action("managed")
            .into();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("TurnstileTask"));
        assert!(json.contains("pageAction"));
    }

    #[test]
    fn test_turnstile_challenge_token_serialization() {
        let task: CapmonsterTask = TurnstileChallenge::token(
            "https://example.com",
            "site-key",
            "managed",
            "data",
            "page-data",
            "Mozilla/5.0",
        )
        .into();

        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("cloudflareTaskType"));
        assert!(json.contains("token"));
        assert!(json.contains("pageData"));
    }

    #[test]
    fn test_turnstile_wait_room_serialization() {
        let proxy = ProxyConfig::http("127.0.0.1", 8080).with_auth("user", "pass");
        let task: CapmonsterTask = TurnstileWaitRoom::new(
            "https://example.com",
            "site-key",
            "base64-html",
            "Mozilla/5.0",
            proxy,
        )
        .into();

        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("wait_room"));
        assert!(json.contains("htmlPageBase64"));
        assert!(json.contains("proxyAddress"));
    }

    #[test]
    fn test_create_task_data_numeric_task_id() {
        let json = r#"{"errorId":0,"taskId":54137240716}"#;
        let data: CreateTaskData = serde_json::from_str(json).unwrap();
        assert_eq!(data.task_id, "54137240716");
    }

    #[test]
    fn test_create_task_data_string_task_id() {
        let json = r#"{"errorId":0,"taskId":"54137240716"}"#;
        let data: CreateTaskData = serde_json::from_str(json).unwrap();
        assert_eq!(data.task_id, "54137240716");
    }

    #[test]
    fn test_image_to_text_serialization() {
        let task: CapmonsterTask = ImageToText::from_base64("aVZCT1J3MEtHZ29B")
            .try_into()
            .unwrap();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("ImageToTextTask"));
        assert!(json.contains("\"body\":\"aVZCT1J3MEtHZ29B\""));
        assert!(!json.contains("CapMonsterModule"));
    }

    #[test]
    fn test_image_to_text_with_module_serialization() {
        let task: CapmonsterTask = ImageToText::from_base64("base64data")
            .with_module("yandex")
            .try_into()
            .unwrap();
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("ImageToTextTask"));
        assert!(json.contains("\"CapMonsterModule\":\"yandex\""));
    }

    #[test]
    fn test_image_to_text_solution_deserialization() {
        let json = r#"{"text": "ABC123"}"#;
        let solution: CapmonsterSolution = serde_json::from_str(json).unwrap();
        let text_solution = solution.into_image_to_text();
        assert_eq!(text_solution.text(), "ABC123");
    }

    #[test]
    fn test_image_to_text_display() {
        let task: CapmonsterTask = ImageToText::from_base64("data").try_into().unwrap();
        assert_eq!(task.to_string(), "ImageToText");
    }

    #[test]
    fn test_recaptcha_v3_rejects_proxy() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        let task = ReCaptchaV3::new("https://example.com", "key").with_proxy(proxy);
        let result: Result<CapmonsterTask, _> = task.try_into();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.unsupported_fields.contains(&"proxy"));
    }

    #[test]
    fn test_recaptcha_v3_rejects_api_domain() {
        let task = ReCaptchaV3::new("https://example.com", "key").with_api_domain("recaptcha.net");
        let result: Result<CapmonsterTask, _> = task.try_into();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.unsupported_fields.contains(&"api_domain"));
    }

    #[test]
    fn test_recaptcha_v3_rejects_enterprise_payload() {
        use std::collections::HashMap;
        let mut payload = HashMap::new();
        payload.insert("s".to_string(), serde_json::json!("value"));
        let task = ReCaptchaV3::new("https://example.com", "key").with_enterprise_payload(payload);
        let result: Result<CapmonsterTask, _> = task.try_into();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.unsupported_fields.contains(&"enterprise_payload"));
    }

    #[test]
    fn test_image_to_text_rejects_ocr_fields() {
        let task = ImageToText::from_base64("data")
            .case_sensitive()
            .numbers_only()
            .with_min_length(4);
        let result: Result<CapmonsterTask, _> = task.try_into();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.unsupported_fields.contains(&"case_sensitive"));
        assert!(err.unsupported_fields.contains(&"numeric"));
        assert!(err.unsupported_fields.contains(&"min_length"));
    }
}

use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Capsolver task types for the API request
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum CapsolverTask {
    /// Cloudflare Turnstile captcha (proxyless)
    AntiTurnstileTaskProxyLess {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
    },
    // Future task types can be added here:
    // ReCaptchaV2TaskProxyLess { ... },
    // ReCaptchaV3TaskProxyLess { ... },
    // HCaptchaTaskProxyLess { ... },
}

impl CapsolverTask {
    /// Create a Turnstile task
    pub fn turnstile(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self::AntiTurnstileTaskProxyLess {
            website_url: website_url.into(),
            website_key: website_key.into(),
        }
    }
}

impl Display for CapsolverTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AntiTurnstileTaskProxyLess { .. } => write!(f, "Turnstile"),
        }
    }
}

/// Capsolver solution types
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CapsolverSolution {
    /// Turnstile solution
    Turnstile(TurnstileSolution),
}

impl CapsolverSolution {
    /// Extract Turnstile solution
    pub fn into_turnstile(self) -> TurnstileSolution {
        match self {
            Self::Turnstile(solution) => solution,
        }
    }

    /// Try to extract Turnstile solution
    pub fn as_turnstile(&self) -> Option<&TurnstileSolution> {
        match self {
            Self::Turnstile(solution) => Some(solution),
        }
    }
}

/// Turnstile captcha solution
#[derive(Debug, Clone, Deserialize)]
pub struct TurnstileSolution {
    pub token: String,
}

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
//! Cloudflare Turnstile Challenge task type.

use crate::utils::proxy::ProxyConfig;

/// Cloudflare Challenge solving mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnstileChallengeMode {
    /// Returns Turnstile token in solution.
    Token,
    /// Returns `cf_clearance` cookie in solution.
    CfClearance,
}

impl TurnstileChallengeMode {
    /// Returns Cloudflare task type value for provider API payload.
    pub fn as_cloudflare_task_type(&self) -> &'static str {
        match self {
            Self::Token => "token",
            Self::CfClearance => "cf_clearance",
        }
    }
}

/// Cloudflare Turnstile Challenge task.
///
/// Supports two modes:
/// - `Token`: returns token (`cloudflareTaskType=token`)
/// - `CfClearance`: returns clearance cookie (`cloudflareTaskType=cf_clearance`)
#[derive(Debug, Clone)]
pub struct TurnstileChallenge {
    /// Full URL of the page with challenge.
    pub website_url: String,
    /// Turnstile site key.
    pub website_key: String,
    /// Challenge mode.
    pub mode: TurnstileChallengeMode,
    /// Action parameter (`managed`, `non-interactive`, etc.) for token mode.
    pub page_action: Option<String>,
    /// `cData` value for token mode.
    pub data: Option<String>,
    /// `chlPageData` value for token mode.
    pub page_data: Option<String>,
    /// Optional API JS URL.
    pub api_js_url: Option<String>,
    /// Base64-encoded challenge HTML page for `cf_clearance` mode.
    pub html_page_base64: Option<String>,
    /// User-Agent used during solving.
    pub user_agent: String,
    /// Optional proxy for token mode, required for cf_clearance mode.
    pub proxy: Option<ProxyConfig>,
}

impl TurnstileChallenge {
    /// Create challenge task in token mode.
    pub fn token(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        page_action: impl Into<String>,
        data: impl Into<String>,
        page_data: impl Into<String>,
        user_agent: impl Into<String>,
    ) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            mode: TurnstileChallengeMode::Token,
            page_action: Some(page_action.into()),
            data: Some(data.into()),
            page_data: Some(page_data.into()),
            api_js_url: None,
            html_page_base64: None,
            user_agent: user_agent.into(),
            proxy: None,
        }
    }

    /// Create challenge task in cf_clearance mode.
    pub fn cf_clearance(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        html_page_base64: impl Into<String>,
        user_agent: impl Into<String>,
        proxy: ProxyConfig,
    ) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            mode: TurnstileChallengeMode::CfClearance,
            page_action: None,
            data: None,
            page_data: None,
            api_js_url: None,
            html_page_base64: Some(html_page_base64.into()),
            user_agent: user_agent.into(),
            proxy: Some(proxy),
        }
    }

    /// Set custom API JS URL.
    pub fn with_api_js_url(mut self, api_js_url: impl Into<String>) -> Self {
        self.api_js_url = Some(api_js_url.into());
        self
    }

    /// Set proxy (useful for token mode).
    pub fn with_proxy(mut self, proxy: ProxyConfig) -> Self {
        self.proxy = Some(proxy);
        self
    }

    /// Returns true if mode is token.
    pub fn is_token_mode(&self) -> bool {
        matches!(self.mode, TurnstileChallengeMode::Token)
    }

    /// Returns true if mode is cf_clearance.
    pub fn is_cf_clearance_mode(&self) -> bool {
        matches!(self.mode, TurnstileChallengeMode::CfClearance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_mode_builder() {
        let task = TurnstileChallenge::token(
            "https://example.com",
            "site-key",
            "managed",
            "cdata",
            "pagedata",
            "Mozilla/5.0",
        );

        assert!(task.is_token_mode());
        assert_eq!(task.page_action.as_deref(), Some("managed"));
        assert_eq!(task.data.as_deref(), Some("cdata"));
        assert_eq!(task.page_data.as_deref(), Some("pagedata"));
        assert!(task.html_page_base64.is_none());
        assert!(task.proxy.is_none());
    }

    #[test]
    fn test_cf_clearance_mode_builder() {
        let proxy = ProxyConfig::http("127.0.0.1", 8080).with_auth("user", "pass");
        let task = TurnstileChallenge::cf_clearance(
            "https://example.com",
            "site-key",
            "base64-html",
            "Mozilla/5.0",
            proxy,
        );

        assert!(task.is_cf_clearance_mode());
        assert_eq!(task.html_page_base64.as_deref(), Some("base64-html"));
        assert!(task.page_action.is_none());
        assert!(task.data.is_none());
        assert!(task.page_data.is_none());
        assert!(task.proxy.is_some());
    }

    #[test]
    fn test_cloudflare_task_type_value() {
        assert_eq!(
            TurnstileChallengeMode::Token.as_cloudflare_task_type(),
            "token"
        );
        assert_eq!(
            TurnstileChallengeMode::CfClearance.as_cloudflare_task_type(),
            "cf_clearance"
        );
    }
}

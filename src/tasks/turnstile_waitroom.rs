//! Cloudflare Waiting Room task type.

use crate::utils::proxy::ProxyConfig;

/// Cloudflare Waiting Room task.
#[derive(Debug, Clone)]
pub struct TurnstileWaitRoom {
    /// Full URL of the waiting room page.
    pub website_url: String,
    /// Site key value.
    pub website_key: String,
    /// Base64-encoded waiting room HTML page.
    pub html_page_base64: String,
    /// User-Agent used during solving.
    pub user_agent: String,
    /// Proxy configuration (required).
    pub proxy: ProxyConfig,
}

impl TurnstileWaitRoom {
    /// Create a new waiting room task.
    pub fn new(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        html_page_base64: impl Into<String>,
        user_agent: impl Into<String>,
        proxy: ProxyConfig,
    ) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            html_page_base64: html_page_base64.into(),
            user_agent: user_agent.into(),
            proxy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waitroom_task_builder() {
        let proxy = ProxyConfig::socks5("proxy.example.com", 1080).with_auth("u", "p");
        let task = TurnstileWaitRoom::new(
            "https://example.com",
            "site-key",
            "base64-html",
            "Mozilla/5.0",
            proxy,
        );

        assert_eq!(task.website_url, "https://example.com");
        assert_eq!(task.website_key, "site-key");
        assert_eq!(task.html_page_base64, "base64-html");
        assert_eq!(task.user_agent, "Mozilla/5.0");
        assert_eq!(task.proxy.address, "proxy.example.com");
        assert_eq!(task.proxy.port, 1080);
    }
}

//! Turnstile and Cloudflare Challenge task types with builder pattern.
//!
//! This module provides provider-agnostic Cloudflare captcha task definitions
//! that can be converted to any supported provider's format.

use crate::proxy::ProxyConfig;

/// Cloudflare Turnstile task with fluent builder pattern.
///
/// Turnstile is Cloudflare's user-friendly captcha alternative. It can work
/// without user interaction in many cases.
///
/// # Examples
///
/// ```
/// use captcha_solvers::tasks::Turnstile;
///
/// // Simple proxyless task
/// let task = Turnstile::new("https://example.com", "0x4AAAAAAAB...");
/// assert!(!task.has_proxy());
///
/// // With action and cdata
/// let task = Turnstile::new("https://example.com", "0x4AAAAAAAB...")
///     .with_action("login")
///     .with_cdata("custom-data");
/// ```
///
/// # Finding the Site Key
///
/// The site key can be found in the page source:
/// - Look for `data-sitekey` attribute on the Turnstile element
/// - Or in JavaScript: `turnstile.render({ sitekey: "..." })`
#[derive(Debug, Clone)]
pub struct Turnstile {
    /// Full URL of the page with the Turnstile widget
    pub website_url: String,
    /// The Turnstile site key (starts with "0x4")
    pub website_key: String,
    /// Action parameter from `data-action` attribute
    pub action: Option<String>,
    /// Custom data from `data-cdata` attribute
    pub cdata: Option<String>,
    /// Page data from `chlPageData` in turnstile.render
    pub pagedata: Option<String>,
    /// Proxy configuration (optional for Turnstile)
    pub proxy: Option<ProxyConfig>,
}

impl Turnstile {
    /// Create a new Turnstile task.
    ///
    /// # Arguments
    ///
    /// * `website_url` - Full URL of the page containing the Turnstile widget
    /// * `website_key` - The Turnstile site key (starts with "0x4")
    ///
    /// # Example
    ///
    /// ```
    /// use captcha_solvers::tasks::Turnstile;
    ///
    /// let task = Turnstile::new(
    ///     "https://example.com/login",
    ///     "0x4AAAAAAABkMYinukE8nV5g"
    /// );
    /// ```
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            action: None,
            cdata: None,
            pagedata: None,
            proxy: None,
        }
    }

    /// Set the action parameter.
    ///
    /// This comes from the `data-action` attribute on the Turnstile element
    /// or the `action` parameter in `turnstile.render()`.
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Set the cdata (custom data) parameter.
    ///
    /// This comes from the `data-cdata` attribute on the Turnstile element
    /// or the `cData` parameter in `turnstile.render()`.
    pub fn with_cdata(mut self, cdata: impl Into<String>) -> Self {
        self.cdata = Some(cdata.into());
        self
    }

    /// Set the pagedata parameter.
    ///
    /// This comes from the `chlPageData` parameter in `turnstile.render()`.
    pub fn with_pagedata(mut self, pagedata: impl Into<String>) -> Self {
        self.pagedata = Some(pagedata.into());
        self
    }

    /// Set the proxy configuration.
    ///
    /// Turnstile can usually be solved without a proxy, but some providers
    /// support proxy-based solving for better success rates.
    pub fn with_proxy(mut self, proxy: ProxyConfig) -> Self {
        self.proxy = Some(proxy);
        self
    }

    /// Check if this task has a proxy configured.
    pub fn has_proxy(&self) -> bool {
        self.proxy.is_some()
    }

    /// Get the website URL.
    pub fn website_url(&self) -> &str {
        &self.website_url
    }

    /// Get the website key.
    pub fn website_key(&self) -> &str {
        &self.website_key
    }

    /// Get the action if set.
    pub fn action(&self) -> Option<&str> {
        self.action.as_deref()
    }

    /// Get the cdata if set.
    pub fn cdata(&self) -> Option<&str> {
        self.cdata.as_deref()
    }

    /// Get the proxy configuration if set.
    pub fn proxy(&self) -> Option<&ProxyConfig> {
        self.proxy.as_ref()
    }
}

/// Cloudflare Challenge task with fluent builder pattern.
///
/// This task type handles full-page Cloudflare challenge bypass (the "Just a moment..."
/// or "Checking your browser" pages). Unlike Turnstile, this always requires a proxy.
///
/// # Important Notes
///
/// - **Proxy Required**: A proxy is always required and must be static or sticky
///   (the same IP throughout the solving process). Rotating proxies will fail.
/// - **User Agent**: You should use the same user agent when making requests
///   with the solved cookies.
/// - **Capsolver Only**: This task type is currently only supported by Capsolver.
///
/// # Examples
///
/// ```
/// use captcha_solvers::{ProxyConfig, tasks::CloudflareChallenge};
///
/// let proxy = ProxyConfig::http("192.168.1.1", 8080);
///
/// // Basic challenge
/// let task = CloudflareChallenge::new("https://example.com", proxy);
///
/// // With user agent for consistency
/// let proxy = ProxyConfig::http("192.168.1.1", 8080);
/// let task = CloudflareChallenge::new("https://example.com", proxy)
///     .with_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)...");
/// ```
#[derive(Debug, Clone)]
pub struct CloudflareChallenge {
    /// Full URL of the page with the Cloudflare challenge
    pub website_url: String,
    /// Proxy configuration (always required)
    pub proxy: ProxyConfig,
    /// User agent to use (should match your actual requests)
    pub user_agent: Option<String>,
    /// Challenge page HTML (for faster solving)
    pub html: Option<String>,
}

impl CloudflareChallenge {
    /// Create a new Cloudflare Challenge task.
    ///
    /// # Arguments
    ///
    /// * `website_url` - Full URL of the protected page
    /// * `proxy` - Proxy configuration (must be static or sticky, not rotating)
    ///
    /// # Example
    ///
    /// ```
    /// use captcha_solvers::{ProxyConfig, tasks::CloudflareChallenge};
    ///
    /// let proxy = ProxyConfig::http("proxy.example.com", 8080)
    ///     .with_auth("user", "pass");
    ///
    /// let task = CloudflareChallenge::new("https://protected-site.com", proxy);
    /// ```
    pub fn new(website_url: impl Into<String>, proxy: ProxyConfig) -> Self {
        Self {
            website_url: website_url.into(),
            proxy,
            user_agent: None,
            html: None,
        }
    }

    /// Set a custom user agent.
    ///
    /// **Important**: Use the same user agent when making subsequent requests
    /// with the solved cookies, otherwise Cloudflare may reject them.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set the challenge HTML.
    ///
    /// This is the HTML response from the target website containing
    /// the "Just a moment..." challenge page. Providing this can speed up solving.
    pub fn with_html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    /// Get the proxy configuration.
    pub fn proxy(&self) -> &ProxyConfig {
        &self.proxy
    }

    /// Get the website URL.
    pub fn website_url(&self) -> &str {
        &self.website_url
    }

    /// Get the user agent if set.
    pub fn user_agent(&self) -> Option<&str> {
        self.user_agent.as_deref()
    }

    /// Get the HTML if set.
    pub fn html(&self) -> Option<&str> {
        self.html.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProxyType;

    // =========================================================================
    // Turnstile Tests
    // =========================================================================

    #[test]
    fn test_turnstile_new() {
        let task = Turnstile::new("https://example.com", "0x4AAAAAAA");

        assert_eq!(task.website_url(), "https://example.com");
        assert_eq!(task.website_key(), "0x4AAAAAAA");
        assert!(!task.has_proxy());
        assert_eq!(task.action(), None);
        assert_eq!(task.cdata(), None);
    }

    #[test]
    fn test_turnstile_with_action() {
        let task = Turnstile::new("https://example.com", "key").with_action("login");

        assert_eq!(task.action(), Some("login"));
    }

    #[test]
    fn test_turnstile_with_cdata() {
        let task = Turnstile::new("https://example.com", "key").with_cdata("custom-data");

        assert_eq!(task.cdata(), Some("custom-data"));
    }

    #[test]
    fn test_turnstile_with_pagedata() {
        let task = Turnstile::new("https://example.com", "key").with_pagedata("page-data");

        assert_eq!(task.pagedata, Some("page-data".to_string()));
    }

    #[test]
    fn test_turnstile_with_proxy() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        let task = Turnstile::new("https://example.com", "key").with_proxy(proxy);

        assert!(task.has_proxy());
        assert_eq!(task.proxy().unwrap().address, "192.168.1.1");
    }

    #[test]
    fn test_turnstile_with_all_options() {
        let proxy = ProxyConfig::socks5("proxy.example.com", 1080);
        let task = Turnstile::new("https://example.com", "0x4AAAAAAA")
            .with_action("submit")
            .with_cdata("custom-data")
            .with_pagedata("page-data")
            .with_proxy(proxy);

        assert_eq!(task.website_url(), "https://example.com");
        assert_eq!(task.website_key(), "0x4AAAAAAA");
        assert_eq!(task.action(), Some("submit"));
        assert_eq!(task.cdata(), Some("custom-data"));
        assert_eq!(task.pagedata, Some("page-data".to_string()));
        assert!(task.has_proxy());
    }

    #[test]
    fn test_turnstile_clone() {
        let task = Turnstile::new("https://example.com", "key").with_action("login");

        let cloned = task.clone();
        assert_eq!(cloned.website_url, task.website_url);
        assert_eq!(cloned.action, task.action);
    }

    // =========================================================================
    // CloudflareChallenge Tests
    // =========================================================================

    #[test]
    fn test_cloudflare_challenge_new() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        let task = CloudflareChallenge::new("https://protected.com", proxy);

        assert_eq!(task.website_url(), "https://protected.com");
        assert_eq!(task.proxy().address, "192.168.1.1");
        assert_eq!(task.proxy().port, 8080);
        assert_eq!(task.user_agent(), None);
        assert_eq!(task.html(), None);
    }

    #[test]
    fn test_cloudflare_challenge_with_user_agent() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        let task =
            CloudflareChallenge::new("https://example.com", proxy).with_user_agent("Mozilla/5.0");

        assert_eq!(task.user_agent(), Some("Mozilla/5.0"));
    }

    #[test]
    fn test_cloudflare_challenge_with_html() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        let html = "<html>Just a moment...</html>";
        let task = CloudflareChallenge::new("https://example.com", proxy).with_html(html);

        assert_eq!(task.html(), Some(html));
    }

    #[test]
    fn test_cloudflare_challenge_with_all_options() {
        let proxy = ProxyConfig::socks5("proxy.example.com", 1080).with_auth("user", "pass");
        let task = CloudflareChallenge::new("https://protected.com", proxy)
            .with_user_agent("Mozilla/5.0 (Windows NT 10.0)")
            .with_html("<html>Challenge page</html>");

        assert_eq!(task.website_url(), "https://protected.com");
        assert_eq!(task.proxy().proxy_type, ProxyType::Socks5);
        assert_eq!(task.proxy().login.as_deref(), Some("user"));
        assert_eq!(task.user_agent(), Some("Mozilla/5.0 (Windows NT 10.0)"));
        assert_eq!(task.html(), Some("<html>Challenge page</html>"));
    }

    #[test]
    fn test_cloudflare_challenge_clone() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        let task =
            CloudflareChallenge::new("https://example.com", proxy).with_user_agent("Mozilla");

        let cloned = task.clone();
        assert_eq!(cloned.website_url, task.website_url);
        assert_eq!(cloned.user_agent, task.user_agent);
    }

    #[test]
    fn test_cloudflare_challenge_proxy_types() {
        // HTTP proxy
        let http_proxy = ProxyConfig::http("192.168.1.1", 8080);
        let task = CloudflareChallenge::new("https://example.com", http_proxy);
        assert_eq!(task.proxy().proxy_type, ProxyType::Http);

        // SOCKS5 proxy
        let socks5_proxy = ProxyConfig::socks5("192.168.1.1", 1080);
        let task = CloudflareChallenge::new("https://example.com", socks5_proxy);
        assert_eq!(task.proxy().proxy_type, ProxyType::Socks5);
    }
}

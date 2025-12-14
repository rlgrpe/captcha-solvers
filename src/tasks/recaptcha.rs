//! ReCaptcha task types with builder pattern.
//!
//! This module provides provider-agnostic ReCaptcha task definitions that can be
//! converted to any supported provider's format using the `Into` trait.

use crate::utils::proxy::ProxyConfig;
use std::collections::HashMap;

/// ReCaptcha V2 task with fluent builder pattern.
///
/// Use this type to create ReCaptcha V2 solving requests that work with any provider.
/// The task can be converted to provider-specific formats using `.into()`.
///
/// # Examples
///
/// ```
/// use captcha_solvers::ReCaptchaV2;
///
/// // Simple proxyless task
/// let task = ReCaptchaV2::new("https://example.com", "6LeIxAcTAAAA...");
/// assert!(!task.is_invisible());
/// assert!(!task.is_enterprise());
///
/// // Invisible reCAPTCHA
/// let task = ReCaptchaV2::new("https://example.com", "6LeIxAcTAAAA...")
///     .invisible();
/// assert!(task.is_invisible());
///
/// // Enterprise with action
/// let task = ReCaptchaV2::new("https://example.com", "6LeIxAcTAAAA...")
///     .enterprise()
///     .with_action("submit");
/// assert!(task.is_enterprise());
/// ```
///
/// # Converting to Provider Format
///
/// ```ignore
/// use captcha_solvers::ReCaptchaV2;
/// use captcha_solvers::providers::capsolver::CapsolverTask;
///
/// let task = ReCaptchaV2::new("https://example.com", "site-key")
///     .invisible()
///     .enterprise();
///
/// // Convert to Capsolver format
/// let capsolver_task: CapsolverTask = task.into();
/// ```
#[derive(Debug, Clone)]
pub struct ReCaptchaV2 {
    /// Full URL of the page with the reCAPTCHA
    pub website_url: String,
    /// The reCAPTCHA site key (data-sitekey attribute)
    pub website_key: String,
    /// Whether this is an invisible reCAPTCHA
    pub is_invisible: bool,
    /// Whether this is an enterprise reCAPTCHA
    pub is_enterprise: bool,
    /// Page action parameter
    pub page_action: Option<String>,
    /// The data-s value (for specific implementations)
    pub recaptcha_data_s_value: Option<String>,
    /// Enterprise payload as key-value pairs
    pub enterprise_payload: Option<HashMap<String, serde_json::Value>>,
    /// Custom API domain (e.g., "recaptcha.net")
    pub api_domain: Option<String>,
    /// User agent to use when solving
    pub user_agent: Option<String>,
    /// Cookies to pass to the solver
    pub cookies: Option<String>,
    /// Proxy configuration (if required)
    pub proxy: Option<ProxyConfig>,
}

impl ReCaptchaV2 {
    /// Create a new ReCaptcha V2 task.
    ///
    /// # Arguments
    ///
    /// * `website_url` - Full URL of the page containing the captcha
    /// * `website_key` - The reCAPTCHA site key (found in data-sitekey attribute)
    ///
    /// # Example
    ///
    /// ```
    /// use captcha_solvers::ReCaptchaV2;
    ///
    /// let task = ReCaptchaV2::new(
    ///     "https://www.google.com/recaptcha/api2/demo",
    ///     "6Le-wvkSAAAAAPBMRTvw0Q4Muexq9bi0DJwx_mJ-"
    /// );
    /// ```
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            is_invisible: false,
            is_enterprise: false,
            page_action: None,
            recaptcha_data_s_value: None,
            enterprise_payload: None,
            api_domain: None,
            user_agent: None,
            cookies: None,
            proxy: None,
        }
    }

    /// Mark this as an invisible reCAPTCHA.
    ///
    /// Invisible reCAPTCHA runs in the background without user interaction.
    pub fn invisible(mut self) -> Self {
        self.is_invisible = true;
        self
    }

    /// Mark this as an enterprise reCAPTCHA.
    ///
    /// Enterprise reCAPTCHA requires different API endpoints and may need
    /// additional enterprise payload parameters.
    pub fn enterprise(mut self) -> Self {
        self.is_enterprise = true;
        self
    }

    /// Set the page action parameter.
    ///
    /// Some reCAPTCHA implementations use an action parameter for verification.
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.page_action = Some(action.into());
        self
    }

    /// Set the data-s value.
    ///
    /// This is used by some specific reCAPTCHA implementations.
    pub fn with_data_s_value(mut self, value: impl Into<String>) -> Self {
        self.recaptcha_data_s_value = Some(value.into());
        self
    }

    /// Set the enterprise payload.
    ///
    /// This automatically marks the task as enterprise.
    pub fn with_enterprise_payload(mut self, payload: HashMap<String, serde_json::Value>) -> Self {
        self.enterprise_payload = Some(payload);
        self.is_enterprise = true;
        self
    }

    /// Set a custom API domain.
    ///
    /// Use this when the reCAPTCHA is loaded from a different domain
    /// (e.g., "recaptcha.net" instead of "google.com").
    pub fn with_api_domain(mut self, domain: impl Into<String>) -> Self {
        self.api_domain = Some(domain.into());
        self
    }

    /// Set a custom user agent for solving.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set cookies to pass to the solver.
    pub fn with_cookies(mut self, cookies: impl Into<String>) -> Self {
        self.cookies = Some(cookies.into());
        self
    }

    /// Set the proxy configuration.
    ///
    /// Some providers require a proxy for certain task types.
    pub fn with_proxy(mut self, proxy: ProxyConfig) -> Self {
        self.proxy = Some(proxy);
        self
    }

    /// Check if this task has a proxy configured.
    pub fn has_proxy(&self) -> bool {
        self.proxy.is_some()
    }

    /// Check if this is an enterprise reCAPTCHA.
    pub fn is_enterprise(&self) -> bool {
        self.is_enterprise
    }

    /// Check if this is an invisible reCAPTCHA.
    pub fn is_invisible(&self) -> bool {
        self.is_invisible
    }

    /// Get the website URL.
    pub fn website_url(&self) -> &str {
        &self.website_url
    }

    /// Get the website key.
    pub fn website_key(&self) -> &str {
        &self.website_key
    }

    /// Get the proxy configuration if set.
    pub fn proxy(&self) -> Option<&ProxyConfig> {
        self.proxy.as_ref()
    }
}

/// ReCaptcha V3 task with fluent builder pattern.
///
/// ReCaptcha V3 returns a score (0.0 to 1.0) indicating how likely the user is human.
/// Unlike V2, it doesn't require user interaction.
///
/// # Examples
///
/// ```
/// use captcha_solvers::ReCaptchaV3;
///
/// // Basic task
/// let task = ReCaptchaV3::new("https://example.com", "6LeIxAcTAAAA...");
///
/// // With action and minimum score
/// let task = ReCaptchaV3::new("https://example.com", "6LeIxAcTAAAA...")
///     .with_action("submit")
///     .with_min_score(0.9);
/// assert_eq!(task.min_score(), Some(0.9));
///
/// // Enterprise version
/// let task = ReCaptchaV3::new("https://example.com", "6LeIxAcTAAAA...")
///     .enterprise()
///     .with_action("login");
/// assert!(task.is_enterprise());
/// ```
#[derive(Debug, Clone)]
pub struct ReCaptchaV3 {
    /// Full URL of the page with the reCAPTCHA
    pub website_url: String,
    /// The reCAPTCHA site key
    pub website_key: String,
    /// Whether this is an enterprise reCAPTCHA
    pub is_enterprise: bool,
    /// The action parameter (e.g., "submit", "login")
    pub page_action: Option<String>,
    /// Minimum score threshold (0.1 to 0.9)
    pub min_score: Option<f32>,
    /// Enterprise payload as key-value pairs
    pub enterprise_payload: Option<HashMap<String, serde_json::Value>>,
    /// Custom API domain
    pub api_domain: Option<String>,
    /// Proxy configuration (if required)
    pub proxy: Option<ProxyConfig>,
}

impl ReCaptchaV3 {
    /// Create a new ReCaptcha V3 task.
    ///
    /// # Arguments
    ///
    /// * `website_url` - Full URL of the page containing the captcha
    /// * `website_key` - The reCAPTCHA site key
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            is_enterprise: false,
            page_action: None,
            min_score: None,
            enterprise_payload: None,
            api_domain: None,
            proxy: None,
        }
    }

    /// Mark this as an enterprise reCAPTCHA.
    pub fn enterprise(mut self) -> Self {
        self.is_enterprise = true;
        self
    }

    /// Set the page action parameter.
    ///
    /// This should match the action used when calling `grecaptcha.execute()`.
    /// Common values: "submit", "login", "register", "homepage".
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.page_action = Some(action.into());
        self
    }

    /// Set the minimum score threshold.
    ///
    /// Common values:
    /// - 0.3 - Low confidence (allows most traffic)
    /// - 0.7 - Medium confidence
    /// - 0.9 - High confidence (stricter filtering)
    ///
    /// # Panics
    ///
    /// Does not panic, but scores outside 0.0-1.0 may cause API errors.
    pub fn with_min_score(mut self, score: f32) -> Self {
        self.min_score = Some(score);
        self
    }

    /// Set the enterprise payload.
    ///
    /// This automatically marks the task as enterprise.
    pub fn with_enterprise_payload(mut self, payload: HashMap<String, serde_json::Value>) -> Self {
        self.enterprise_payload = Some(payload);
        self.is_enterprise = true;
        self
    }

    /// Set a custom API domain.
    pub fn with_api_domain(mut self, domain: impl Into<String>) -> Self {
        self.api_domain = Some(domain.into());
        self
    }

    /// Set the proxy configuration.
    pub fn with_proxy(mut self, proxy: ProxyConfig) -> Self {
        self.proxy = Some(proxy);
        self
    }

    /// Check if this task has a proxy configured.
    pub fn has_proxy(&self) -> bool {
        self.proxy.is_some()
    }

    /// Check if this is an enterprise reCAPTCHA.
    pub fn is_enterprise(&self) -> bool {
        self.is_enterprise
    }

    /// Get the minimum score threshold if set.
    pub fn min_score(&self) -> Option<f32> {
        self.min_score
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
        self.page_action.as_deref()
    }

    /// Get the proxy configuration if set.
    pub fn proxy(&self) -> Option<&ProxyConfig> {
        self.proxy.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // ReCaptchaV2 Tests
    // =========================================================================

    #[test]
    fn test_recaptcha_v2_new() {
        let task = ReCaptchaV2::new("https://example.com", "site-key");

        assert_eq!(task.website_url(), "https://example.com");
        assert_eq!(task.website_key(), "site-key");
        assert!(!task.is_invisible());
        assert!(!task.is_enterprise());
        assert!(!task.has_proxy());
    }

    #[test]
    fn test_recaptcha_v2_invisible() {
        let task = ReCaptchaV2::new("https://example.com", "site-key").invisible();

        assert!(task.is_invisible());
        assert!(!task.is_enterprise());
    }

    #[test]
    fn test_recaptcha_v2_enterprise() {
        let task = ReCaptchaV2::new("https://example.com", "site-key").enterprise();

        assert!(!task.is_invisible());
        assert!(task.is_enterprise());
    }

    #[test]
    fn test_recaptcha_v2_invisible_enterprise() {
        let task = ReCaptchaV2::new("https://example.com", "site-key")
            .invisible()
            .enterprise();

        assert!(task.is_invisible());
        assert!(task.is_enterprise());
    }

    #[test]
    fn test_recaptcha_v2_with_action() {
        let task = ReCaptchaV2::new("https://example.com", "site-key").with_action("submit");

        assert_eq!(task.page_action, Some("submit".to_string()));
    }

    #[test]
    fn test_recaptcha_v2_with_enterprise_payload() {
        let mut payload = HashMap::new();
        payload.insert("s".to_string(), serde_json::json!("SOME_TOKEN"));

        let task = ReCaptchaV2::new("https://example.com", "site-key")
            .with_enterprise_payload(payload.clone());

        assert!(task.is_enterprise()); // Auto-set
        assert_eq!(task.enterprise_payload, Some(payload));
    }

    #[test]
    fn test_recaptcha_v2_with_proxy() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);
        let task = ReCaptchaV2::new("https://example.com", "site-key").with_proxy(proxy.clone());

        assert!(task.has_proxy());
        assert_eq!(task.proxy().unwrap().address, "192.168.1.1");
        assert_eq!(task.proxy().unwrap().port, 8080);
    }

    #[test]
    fn test_recaptcha_v2_with_all_options() {
        let proxy = ProxyConfig::socks5("proxy.example.com", 1080).with_auth("user", "pass");

        let task = ReCaptchaV2::new("https://example.com", "site-key")
            .invisible()
            .enterprise()
            .with_action("verify")
            .with_api_domain("recaptcha.net")
            .with_user_agent("Mozilla/5.0")
            .with_cookies("session=abc123")
            .with_proxy(proxy);

        assert!(task.is_invisible());
        assert!(task.is_enterprise());
        assert_eq!(task.page_action, Some("verify".to_string()));
        assert_eq!(task.api_domain, Some("recaptcha.net".to_string()));
        assert_eq!(task.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(task.cookies, Some("session=abc123".to_string()));
        assert!(task.has_proxy());
    }

    #[test]
    fn test_recaptcha_v2_clone() {
        let task = ReCaptchaV2::new("https://example.com", "site-key")
            .invisible()
            .enterprise();

        let cloned = task.clone();
        assert_eq!(cloned.website_url, task.website_url);
        assert_eq!(cloned.is_invisible, task.is_invisible);
        assert_eq!(cloned.is_enterprise, task.is_enterprise);
    }

    // =========================================================================
    // ReCaptchaV3 Tests
    // =========================================================================

    #[test]
    fn test_recaptcha_v3_new() {
        let task = ReCaptchaV3::new("https://example.com", "site-key");

        assert_eq!(task.website_url(), "https://example.com");
        assert_eq!(task.website_key(), "site-key");
        assert!(!task.is_enterprise());
        assert!(!task.has_proxy());
        assert_eq!(task.min_score(), None);
        assert_eq!(task.action(), None);
    }

    #[test]
    fn test_recaptcha_v3_with_action() {
        let task = ReCaptchaV3::new("https://example.com", "site-key").with_action("login");

        assert_eq!(task.action(), Some("login"));
    }

    #[test]
    fn test_recaptcha_v3_with_min_score() {
        let task = ReCaptchaV3::new("https://example.com", "site-key").with_min_score(0.9);

        assert_eq!(task.min_score(), Some(0.9));
    }

    #[test]
    fn test_recaptcha_v3_enterprise() {
        let task = ReCaptchaV3::new("https://example.com", "site-key").enterprise();

        assert!(task.is_enterprise());
    }

    #[test]
    fn test_recaptcha_v3_with_all_options() {
        let proxy = ProxyConfig::http("192.168.1.1", 8080);

        let task = ReCaptchaV3::new("https://example.com", "site-key")
            .enterprise()
            .with_action("submit")
            .with_min_score(0.7)
            .with_api_domain("recaptcha.net")
            .with_proxy(proxy);

        assert!(task.is_enterprise());
        assert_eq!(task.action(), Some("submit"));
        assert_eq!(task.min_score(), Some(0.7));
        assert_eq!(task.api_domain, Some("recaptcha.net".to_string()));
        assert!(task.has_proxy());
    }

    #[test]
    fn test_recaptcha_v3_common_scores() {
        // Test common score values
        let low = ReCaptchaV3::new("https://example.com", "key").with_min_score(0.3);
        let medium = ReCaptchaV3::new("https://example.com", "key").with_min_score(0.7);
        let high = ReCaptchaV3::new("https://example.com", "key").with_min_score(0.9);

        assert_eq!(low.min_score(), Some(0.3));
        assert_eq!(medium.min_score(), Some(0.7));
        assert_eq!(high.min_score(), Some(0.9));
    }
}

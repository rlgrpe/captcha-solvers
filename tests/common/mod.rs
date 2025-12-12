//! Common helpers for integration tests.

use std::env;

/// Load environment variables from .env file
pub fn load_env() {
    let _ = dotenvy::dotenv();
}

/// Get Capsolver API key from environment
pub fn capsolver_api_key() -> Option<String> {
    load_env();
    env::var("CAPSOLVER_API_KEY").ok()
}

/// Get RuCaptcha API key from environment
pub fn rucaptcha_api_key() -> Option<String> {
    load_env();
    env::var("RUCAPTCHA_API_KEY").ok()
}

/// Get proxy configuration from environment
///
/// Expected environment variables:
/// - `PROXY_HOST` - Proxy hostname or IP
/// - `PROXY_PORT` - Proxy port number
/// - `PROXY_TYPE` - One of: http, https, socks4, socks5 (default: http)
/// - `PROXY_USER` - Optional proxy username
/// - `PROXY_PASSWORD` - Optional proxy password
pub fn proxy_config() -> Option<captcha_solvers::ProxyConfig> {
    load_env();

    let host = env::var("PROXY_HOST").ok()?;
    let port: u16 = env::var("PROXY_PORT").ok()?.parse().ok()?;
    let proxy_type = env::var("PROXY_TYPE").ok().unwrap_or_else(|| "http".to_string());

    let proxy = match proxy_type.to_lowercase().as_str() {
        "https" => captcha_solvers::ProxyConfig::https(&host, port),
        "socks4" => captcha_solvers::ProxyConfig::socks4(&host, port),
        "socks5" => captcha_solvers::ProxyConfig::socks5(&host, port),
        _ => captcha_solvers::ProxyConfig::http(&host, port),
    };

    // Add auth if provided
    let proxy = match (env::var("PROXY_USER").ok(), env::var("PROXY_PASSWORD").ok()) {
        (Some(user), Some(pass)) => proxy.with_auth(user, pass),
        _ => proxy,
    };

    Some(proxy)
}

/// Skip test if API key is not set
#[macro_export]
macro_rules! skip_if_no_api_key {
    ($key:expr) => {
        if $key.is_none() {
            eprintln!("Skipping test: API key not set");
            return;
        }
    };
}

/// Skip test if proxy is not set
#[macro_export]
macro_rules! skip_if_no_proxy {
    ($proxy:expr) => {
        if $proxy.is_none() {
            eprintln!("Skipping test: Proxy not configured");
            return;
        }
    };
}

// ============================================================================
// Test Fixtures
// ============================================================================

/// Sample website URL for tests
pub const TEST_WEBSITE_URL: &str = "https://example.com";

/// Sample site key for tests
pub const TEST_SITE_KEY: &str = "6LcR_RsTAAAAAFPjPHX8DQAKcmPqzBgP1n3f_example";

/// Create a sample HTTP proxy for tests
pub fn sample_http_proxy() -> captcha_solvers::ProxyConfig {
    captcha_solvers::ProxyConfig::http("192.168.1.1", 8080)
}

/// Create a sample SOCKS5 proxy with authentication for tests
pub fn sample_socks5_proxy_with_auth() -> captcha_solvers::ProxyConfig {
    captcha_solvers::ProxyConfig::socks5("192.168.1.1", 1080).with_auth("user", "pass")
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert that a JSON string contains a specific field with a specific value
pub fn assert_json_contains(json: &str, field: &str, value: &str) {
    assert!(
        json.contains(&format!("\"{}\"", field)),
        "JSON should contain field '{}': {}",
        field,
        json
    );
    assert!(
        json.contains(value),
        "JSON should contain value '{}': {}",
        value,
        json
    );
}

/// Assert that a JSON string contains a specific task type
pub fn assert_task_type(json: &str, expected_type: &str) {
    assert!(
        json.contains(&format!("\"type\":\"{}\"", expected_type)),
        "JSON should contain task type '{}': {}",
        expected_type,
        json
    );
}

/// Assert that a JSON string contains proxy fields
pub fn assert_has_proxy_fields(json: &str) {
    assert!(json.contains("proxyType"), "JSON should contain proxyType: {}", json);
    assert!(json.contains("proxyAddress"), "JSON should contain proxyAddress: {}", json);
    assert!(json.contains("proxyPort"), "JSON should contain proxyPort: {}", json);
}

/// Assert that a JSON string does NOT contain proxy fields
pub fn assert_no_proxy_fields(json: &str) {
    assert!(!json.contains("proxyType"), "JSON should NOT contain proxyType: {}", json);
    assert!(!json.contains("proxyAddress"), "JSON should NOT contain proxyAddress: {}", json);
}

// ============================================================================
// Shared Task Helpers
// ============================================================================

pub mod shared {
    use super::*;
    use captcha_solvers::tasks::{ReCaptchaV2, ReCaptchaV3, Turnstile, CloudflareChallenge};

    /// Create a sample shared ReCaptcha V2 task
    pub fn sample_recaptcha_v2() -> ReCaptchaV2 {
        ReCaptchaV2::new(TEST_WEBSITE_URL, TEST_SITE_KEY)
    }

    /// Create a sample shared ReCaptcha V2 invisible task
    pub fn sample_recaptcha_v2_invisible() -> ReCaptchaV2 {
        ReCaptchaV2::new(TEST_WEBSITE_URL, TEST_SITE_KEY).invisible()
    }

    /// Create a sample shared ReCaptcha V2 enterprise task
    pub fn sample_recaptcha_v2_enterprise() -> ReCaptchaV2 {
        ReCaptchaV2::new(TEST_WEBSITE_URL, TEST_SITE_KEY).enterprise()
    }

    /// Create a sample shared ReCaptcha V2 task with proxy
    pub fn sample_recaptcha_v2_with_proxy() -> ReCaptchaV2 {
        ReCaptchaV2::new(TEST_WEBSITE_URL, TEST_SITE_KEY)
            .with_proxy(sample_http_proxy())
    }

    /// Create a sample shared ReCaptcha V3 task
    pub fn sample_recaptcha_v3() -> ReCaptchaV3 {
        ReCaptchaV3::new(TEST_WEBSITE_URL, TEST_SITE_KEY)
    }

    /// Create a sample shared ReCaptcha V3 task with action
    pub fn sample_recaptcha_v3_with_action() -> ReCaptchaV3 {
        ReCaptchaV3::new(TEST_WEBSITE_URL, TEST_SITE_KEY)
            .with_action("submit")
            .with_min_score(0.9)
    }

    /// Create a sample shared Turnstile task
    pub fn sample_turnstile() -> Turnstile {
        Turnstile::new(TEST_WEBSITE_URL, TEST_SITE_KEY)
    }

    /// Create a sample shared Turnstile task with metadata
    pub fn sample_turnstile_with_metadata() -> Turnstile {
        Turnstile::new(TEST_WEBSITE_URL, TEST_SITE_KEY)
            .with_action("login")
            .with_cdata("custom-data")
    }

    /// Create a sample shared Cloudflare Challenge task
    pub fn sample_cloudflare_challenge() -> CloudflareChallenge {
        CloudflareChallenge::new(TEST_WEBSITE_URL, sample_http_proxy())
    }
}

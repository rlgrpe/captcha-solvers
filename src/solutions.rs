//! Shared solution types for all captcha solving providers.
//!
//! These types represent the solutions returned by captcha solving services.
//! They are designed to work with all supported providers while capturing
//! provider-specific fields where applicable.

use serde::Deserialize;
use std::collections::HashMap;

/// Marker trait for provider solution types.
///
/// All provider-specific solution types (like `CapsolverSolution`, `RucaptchaSolution`)
/// must implement this trait. It provides a common bound for the service trait.
///
/// This trait is automatically implemented for solution types that are `Send + Sync`.
pub trait ProviderSolution: Send + Sync {}

/// ReCaptcha solution (V2 and V3)
///
/// This solution type is returned when solving ReCaptcha V2 or V3 captchas.
/// The primary field is `g_recaptcha_response` which contains the token to submit.
///
/// # Example
///
/// ```ignore
/// let solution = service.solve_captcha(task, timeout).await?;
/// let recaptcha = solution.into_recaptcha();
/// println!("Token: {}", recaptcha.token());
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReCaptchaSolution {
    /// The reCAPTCHA token (required field)
    #[serde(rename = "gRecaptchaResponse")]
    pub g_recaptcha_response: String,

    /// Alias for g_recaptcha_response (some providers return this)
    #[serde(default)]
    pub token: Option<String>,

    /// User-Agent string used during solving
    #[serde(default)]
    pub user_agent: Option<String>,

    /// Sec-Ch-Ua header value (Capsolver)
    #[serde(default, rename = "secChUa")]
    pub sec_ch_ua: Option<String>,

    /// Token creation timestamp (Capsolver)
    #[serde(default)]
    pub create_time: Option<u64>,

    /// Session cookie for V3 when isSession is enabled (Capsolver)
    #[serde(default, rename = "recaptcha-ca-t")]
    pub recaptcha_ca_t: Option<String>,

    /// Cookie for some V2 websites (Capsolver)
    #[serde(default, rename = "recaptcha-ca-e")]
    pub recaptcha_ca_e: Option<String>,
}

impl ReCaptchaSolution {
    /// Get the reCAPTCHA token
    ///
    /// This is the value to submit in the `g-recaptcha-response` field.
    pub fn token(&self) -> &str {
        &self.g_recaptcha_response
    }

    /// Get the session cookie (for V3 with isSession enabled)
    pub fn session_cookie(&self) -> Option<&str> {
        self.recaptcha_ca_t.as_deref()
    }
}

/// Turnstile/Cloudflare Challenge solution
///
/// This solution type is returned when solving Cloudflare Turnstile or
/// Cloudflare Challenge captchas. The Cloudflare Challenge also includes
/// cookies needed for subsequent requests.
///
/// # Example
///
/// ```ignore
/// let solution = service.solve_captcha(task, timeout).await?;
/// let turnstile = solution.into_turnstile();
/// println!("Token: {}", turnstile.token());
///
/// // For Cloudflare Challenge, also use the cookies:
/// if let Some(clearance) = turnstile.cf_clearance() {
///     println!("cf_clearance: {}", clearance);
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnstileSolution {
    /// The solved token (Turnstile token or cf_clearance token)
    pub token: String,

    /// Cookies map containing cf_clearance (Cloudflare Challenge only)
    #[serde(default)]
    pub cookies: Option<HashMap<String, String>>,

    /// User-Agent string used (must match your subsequent requests)
    #[serde(default)]
    pub user_agent: Option<String>,
}

impl TurnstileSolution {
    /// Get the token
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Get the cf_clearance cookie value (Cloudflare Challenge only)
    ///
    /// This cookie must be included in subsequent requests to bypass
    /// the Cloudflare protection.
    pub fn cf_clearance(&self) -> Option<&str> {
        self.cookies
            .as_ref()
            .and_then(|c| c.get("cf_clearance").map(|s| s.as_str()))
    }

    /// Get all cookies (Cloudflare Challenge only)
    pub fn cookies(&self) -> Option<&HashMap<String, String>> {
        self.cookies.as_ref()
    }
}

/// Type alias for backwards compatibility
pub type CloudflareChallengeSolution = TurnstileSolution;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recaptcha_solution_full_deserialization() {
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
    fn test_recaptcha_solution_minimal_deserialization() {
        let json = r#"{"gRecaptchaResponse": "token-value"}"#;
        let solution: ReCaptchaSolution = serde_json::from_str(json).unwrap();
        assert_eq!(solution.token(), "token-value");
        assert!(solution.user_agent.is_none());
    }

    #[test]
    fn test_recaptcha_solution_with_token_alias() {
        let json = r#"{"gRecaptchaResponse": "token-value", "token": "token-value"}"#;
        let solution: ReCaptchaSolution = serde_json::from_str(json).unwrap();
        assert_eq!(solution.token(), "token-value");
    }

    #[test]
    fn test_turnstile_solution_deserialization() {
        let json = r#"{"token": "turnstile-token", "userAgent": "Mozilla/5.0"}"#;
        let solution: TurnstileSolution = serde_json::from_str(json).unwrap();
        assert_eq!(solution.token(), "turnstile-token");
        assert_eq!(solution.user_agent.as_deref(), Some("Mozilla/5.0"));
    }

    #[test]
    fn test_cloudflare_solution_with_cookies() {
        let json = r#"{
            "token": "cf-token",
            "cookies": {"cf_clearance": "clearance-value"},
            "userAgent": "Mozilla/5.0"
        }"#;
        let solution: CloudflareChallengeSolution = serde_json::from_str(json).unwrap();
        assert_eq!(solution.token(), "cf-token");
        assert_eq!(solution.cf_clearance(), Some("clearance-value"));
    }
}

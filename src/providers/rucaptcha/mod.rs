//! # RuCaptcha Provider
//!
//! Implementation of the captcha solver provider for [RuCaptcha](https://rucaptcha.com).
//!
//! ## Supported Captcha Types
//!
//! | Type | Task Type | Proxy Required |
//! |------|-----------|----------------|
//! | ReCaptcha V2 | [`ReCaptchaV2`](crate::ReCaptchaV2) | No |
//! | ReCaptcha V2 Invisible | [`ReCaptchaV2`](crate::ReCaptchaV2) with `.invisible()` | No |
//! | ReCaptcha V2 Enterprise | [`ReCaptchaV2`](crate::ReCaptchaV2) with `.enterprise()` | No |
//! | ReCaptcha V3 | [`ReCaptchaV3`](crate::ReCaptchaV3) | No |
//! | ReCaptcha V3 Enterprise | [`ReCaptchaV3`](crate::ReCaptchaV3) with `.enterprise()` | No |
//! | Cloudflare Turnstile | [`Turnstile`](crate::Turnstile) | No |
//!
//! **Note**: [`CloudflareChallenge`](crate::CloudflareChallenge) is not supported by RuCaptcha.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use captcha_solvers::providers::rucaptcha::RucaptchaProvider;
//! use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait, ReCaptchaV2};
//! use std::time::Duration;
//!
//! // Create provider with API key
//! let provider = RucaptchaProvider::new("your_api_key")?;
//! let service = CaptchaSolverService::with_provider(provider);
//!
//! // Solve ReCaptcha V2 using shared task types
//! let task = ReCaptchaV2::new("https://example.com", "site_key")
//!     .invisible()
//!     .enterprise();
//! let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;
//! let token = solution.into_recaptcha().token();
//! ```
//!
//! ## Provider Configuration
//!
//! The provider can be configured using the builder pattern:
//!
//! ```rust,ignore
//! use captcha_solvers::providers::rucaptcha::RucaptchaProvider;
//! use url::Url;
//!
//! // Simple: default API URL
//! let provider = RucaptchaProvider::new("api_key")?;
//!
//! // Custom URL
//! let provider = RucaptchaProvider::with_url(
//!     Url::parse("https://api.rucaptcha.com")?,
//!     "api_key"
//! )?;
//!
//! // Full builder
//! let provider = RucaptchaProvider::builder("api_key")
//!     .url(custom_url)
//!     .http_client(custom_middleware_client)
//!     .build()?;
//! ```
//!
//! ## Using Proxies
//!
//! For tasks that require custom proxies:
//!
//! ```rust,ignore
//! use captcha_solvers::{ReCaptchaV2, ProxyConfig};
//!
//! // HTTP proxy
//! let proxy = ProxyConfig::http("192.168.1.1", 8080);
//!
//! // SOCKS5 proxy with authentication
//! let proxy = ProxyConfig::socks5("proxy.example.com", 1080)
//!     .with_auth("username", "password");
//!
//! // Create task with proxy
//! let task = ReCaptchaV2::new("https://example.com", "site_key")
//!     .with_proxy(proxy);
//! ```
//!
//! ## Solution Types
//!
//! Each captcha type returns a specific solution:
//!
//! - **ReCaptcha V2/V3**: [`ReCaptchaSolution`] with `token()` method
//! - **Turnstile**: [`TurnstileSolution`] with `token()` method
//!
//! ## Error Handling
//!
//! Errors are categorized as retryable or permanent:
//!
//! ```rust,ignore
//! use captcha_solvers::RetryableError;
//! use captcha_solvers::providers::rucaptcha::RucaptchaError;
//!
//! match result {
//!     Err(e) if e.is_retryable() => { /* retry later */ }
//!     Err(e) => { /* permanent error, check API key or task data */ }
//!     Ok(solution) => { /* success */ }
//! }
//! ```

mod errors;
mod provider;
mod response;
mod types;

#[cfg(test)]
mod tests;

// Errors
pub use errors::{RucaptchaApiError, RucaptchaError, RucaptchaErrorCode};

// Provider
pub use provider::{DEFAULT_API_URL, RucaptchaProvider, RucaptchaProviderBuilder};

// Solutions (public API)
pub use types::{ReCaptchaSolution, RucaptchaSolution, TurnstileSolution};

// Re-export proxy types for convenience (also available at crate root)
pub use crate::utils::proxy::{ProxyConfig, ProxyType};

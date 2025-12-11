//! # RuCaptcha Provider
//!
//! Implementation of the captcha solver provider for [RuCaptcha](https://rucaptcha.com).
//!
//! ## Supported Captcha Types
//!
//! | Type | Task Constructor | Proxy Required |
//! |------|-----------------|----------------|
//! | ReCaptcha V2 | `RucaptchaTask::recaptcha_v2()` | No |
//! | ReCaptcha V2 Invisible | `RucaptchaTask::recaptcha_v2_invisible()` | No |
//! | ReCaptcha V2 | `RucaptchaTask::recaptcha_v2_with_proxy()` | Yes |
//! | ReCaptcha V2 Enterprise | `RucaptchaTask::recaptcha_v2_enterprise()` | No |
//! | ReCaptcha V2 Enterprise | `RucaptchaTask::recaptcha_v2_enterprise_with_proxy()` | Yes |
//! | ReCaptcha V3 | `RucaptchaTask::recaptcha_v3()` | No |
//! | ReCaptcha V3 | `RucaptchaTask::recaptcha_v3_with_action()` | No |
//! | ReCaptcha V3 Enterprise | `RucaptchaTask::recaptcha_v3_enterprise()` | No |
//! | Cloudflare Turnstile | `RucaptchaTask::turnstile()` | No |
//! | Cloudflare Turnstile | `RucaptchaTask::turnstile_with_metadata()` | No |
//! | Cloudflare Turnstile | `RucaptchaTask::turnstile_with_proxy()` | Yes |
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use captcha_solvers::providers::rucaptcha::{RucaptchaClient, RucaptchaProvider, RucaptchaTask};
//! use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait};
//! use std::time::Duration;
//!
//! // Create client with default API URL
//! let client = RucaptchaClient::new("your_api_key")?;
//! let provider = RucaptchaProvider::new(client);
//! let service = CaptchaSolverService::with_provider(provider);
//!
//! // Solve ReCaptcha V2
//! let task = RucaptchaTask::recaptcha_v2("https://example.com", "site_key");
//! let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;
//! let token = solution.into_recaptcha().token();
//! ```
//!
//! ## Client Configuration
//!
//! The client can be configured using the builder pattern:
//!
//! ```rust,ignore
//! use captcha_solvers::providers::rucaptcha::RucaptchaClient;
//! use url::Url;
//!
//! // Simple: default API URL
//! let client = RucaptchaClient::new("api_key")?;
//!
//! // Custom URL
//! let client = RucaptchaClient::with_url(
//!     Url::parse("https://api.rucaptcha.com")?,
//!     "api_key"
//! )?;
//!
//! // Full builder
//! let client = RucaptchaClient::builder("api_key")
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
//! use captcha_solvers::providers::rucaptcha::{RucaptchaTask, ProxyConfig, ProxyType};
//!
//! // HTTP proxy
//! let proxy = ProxyConfig::http("192.168.1.1", 8080);
//!
//! // SOCKS5 proxy with authentication
//! let proxy = ProxyConfig::socks5("proxy.example.com", 1080)
//!     .with_auth("username", "password");
//!
//! // Create task with proxy
//! let task = RucaptchaTask::recaptcha_v2_with_proxy(
//!     "https://example.com",
//!     "site_key",
//!     proxy
//! );
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

mod client;
mod errors;
mod provider;
mod response;
mod types;

#[cfg(test)]
mod tests;

// Client
pub use client::{RucaptchaClient, RucaptchaClientBuilder, DEFAULT_API_URL};

// Errors
pub use errors::{RucaptchaApiError, RucaptchaError, RucaptchaErrorCode};

// Provider
pub use provider::RucaptchaProvider;

// Tasks
pub use types::{ProxyConfig, ProxyType, RucaptchaTask, TurnstileMetadata};

// Solutions
pub use types::{ReCaptchaSolution, RucaptchaSolution, TurnstileSolution};
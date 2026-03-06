//! # CapMonster Cloud Provider
//!
//! Implementation of captcha solver provider for [CapMonster Cloud](https://capmonster.cloud).
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
//! | Turnstile Challenge | [`TurnstileChallenge`](crate::TurnstileChallenge) | Depends on mode |
//! | Turnstile Wait Room | [`TurnstileWaitRoom`](crate::TurnstileWaitRoom) | Yes |
//! | Image to Text | [`ImageToText`](crate::ImageToText) | No |
//!
//! **Note**: [`CloudflareChallenge`](crate::CloudflareChallenge) is not supported by CapMonster.
//! For `cf_clearance` use cases, use [`TurnstileChallenge::cf_clearance()`](crate::TurnstileChallenge::cf_clearance)
//! which maps to CapMonster's `TurnstileTask` with `cloudflareTaskType = "cf_clearance"`.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use captcha_solvers::capmonster::CapmonsterProvider;
//! use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait, ReCaptchaV2};
//!
//! let provider = CapmonsterProvider::new("your_api_key")?;
//! let service = CaptchaSolverService::new(provider);
//!
//! let task = ReCaptchaV2::new("https://example.com", "site_key");
//! let solution = service.solve_captcha(task).await?;
//! let token = solution.into_recaptcha().token();
//! println!("Token: {}", token);
//! ```
//!
//! ## Provider Configuration
//!
//! ```rust,ignore
//! use captcha_solvers::capmonster::CapmonsterProvider;
//! use url::Url;
//!
//! // Simple: default API URL
//! let provider = CapmonsterProvider::new("api_key")?;
//!
//! // Custom URL
//! let provider = CapmonsterProvider::with_url(
//!     Url::parse("https://api.capmonster.cloud")?,
//!     "api_key"
//! )?;
//!
//! // Full builder
//! let provider = CapmonsterProvider::builder("api_key")
//!     .url(custom_url)
//!     .http_client(custom_middleware_client)
//!     .build()?;
//! ```
//!
//! ## Turnstile Challenge Examples
//!
//! CapMonster supports two Turnstile Challenge modes via [`TurnstileChallenge`](crate::TurnstileChallenge):
//!
//! ```rust,ignore
//! use captcha_solvers::{TurnstileChallenge, ProxyConfig};
//!
//! // Token mode — returns a Turnstile token
//! let task = TurnstileChallenge::token(
//!     "https://example.com",
//!     "site-key",
//!     "managed",       // page_action
//!     "cdata-value",   // data
//!     "page-data",     // page_data
//!     "Mozilla/5.0...",
//! );
//! let solution = service.solve_captcha(task).await?;
//! let token = solution.into_turnstile().token().unwrap();
//!
//! // cf_clearance mode — returns cf_clearance cookie (requires proxy)
//! let proxy = ProxyConfig::http("192.168.1.1", 8080).with_auth("user", "pass");
//! let task = TurnstileChallenge::cf_clearance(
//!     "https://example.com",
//!     "site-key",
//!     "base64-encoded-html-page",
//!     "Mozilla/5.0...",
//!     proxy,
//! );
//! let solution = service.solve_captcha(task).await?;
//! let clearance = solution.into_turnstile().cf_clearance().unwrap();
//! ```
//!
//! ## Wait Room Example
//!
//! ```rust,ignore
//! use captcha_solvers::{TurnstileWaitRoom, ProxyConfig};
//!
//! let proxy = ProxyConfig::socks5("proxy.example.com", 1080).with_auth("user", "pass");
//! let task = TurnstileWaitRoom::new(
//!     "https://example.com/waitroom",
//!     "site-key",
//!     "base64-encoded-html-page",
//!     "Mozilla/5.0...",
//!     proxy,
//! );
//! let solution = service.solve_captcha(task).await?;
//! let clearance = solution.into_turnstile().cf_clearance().unwrap();
//! ```
//!
//! ## Image to Text Example
//!
//! ```rust,ignore
//! use captcha_solvers::ImageToText;
//!
//! let task = ImageToText::from_base64("iVBORw0KGgoAAAANSUhEUgAA...")
//!     .with_module("yandex");  // Optional CapMonster recognition module
//!
//! let solution = service.solve_captcha(task).await?;
//! let text = solution.into_image_to_text().text();
//! println!("Recognized: {}", text);
//! ```
//!
//! ## Solution Types
//!
//! Each captcha type returns a specific solution:
//!
//! - **ReCaptcha V2/V3**: [`ReCaptchaSolution`] with `token()` method
//! - **Turnstile / Challenge / WaitRoom**: [`TurnstileSolution`] with `token()` and `cf_clearance()` methods
//! - **Image to Text**: [`ImageToTextSolution`] with `text()` method
//!
//! ## Error Handling
//!
//! Errors are categorized as retryable or permanent:
//!
//! ```rust,ignore
//! use captcha_solvers::RetryableError;
//! use captcha_solvers::capmonster::CapmonsterError;
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
pub use errors::{CapmonsterApiError, CapmonsterError, CapmonsterErrorCode};

// Provider
pub use provider::{CapmonsterProvider, CapmonsterProviderBuilder, DEFAULT_API_URL};

// Solutions
pub use types::{CapmonsterSolution, ImageToTextSolution, ReCaptchaSolution, TurnstileSolution};

// Re-export proxy types for convenience (also available at crate root)
pub use crate::utils::proxy::{ProxyConfig, ProxyType};

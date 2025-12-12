//! # Capsolver Provider
//!
//! Implementation of the captcha solver provider for [Capsolver](https://capsolver.com).
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
//! | Cloudflare Challenge | [`CloudflareChallenge`](crate::CloudflareChallenge) | Yes |
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use captcha_solvers::providers::capsolver::CapsolverProvider;
//! use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait, ReCaptchaV2};
//! use std::time::Duration;
//!
//! // Create provider with API key
//! let provider = CapsolverProvider::new("your_api_key")?;
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
//! use captcha_solvers::providers::capsolver::CapsolverProvider;
//! use url::Url;
//!
//! // Simple: default API URL
//! let provider = CapsolverProvider::new("api_key")?;
//!
//! // Custom URL
//! let provider = CapsolverProvider::with_url(
//!     Url::parse("https://api.capsolver.com")?,
//!     "api_key"
//! )?;
//!
//! // Full builder
//! let provider = CapsolverProvider::builder("api_key")
//!     .url(custom_url)
//!     .http_client(custom_middleware_client)
//!     .build()?;
//! ```
//!
//! ## Solution Types
//!
//! Each captcha type returns a specific solution:
//!
//! - **ReCaptcha V2/V3**: [`ReCaptchaSolution`] with `token()` method
//! - **Turnstile**: [`TurnstileSolution`] with `token()` method
//! - **Cloudflare Challenge**: [`CloudflareChallengeSolution`] with `token()` and `cf_clearance()` methods
//!
//! ## Error Handling
//!
//! Errors are categorized as retryable or permanent:
//!
//! ```rust,ignore
//! use captcha_solvers::RetryableError;
//! use captcha_solvers::providers::capsolver::CapsolverError;
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
pub use errors::{CapsolverApiError, CapsolverError, CapsolverErrorCode};

// Provider
pub use provider::{CapsolverProvider, CapsolverProviderBuilder, DEFAULT_API_URL};

// Solutions (public API)
pub use types::{
    CapsolverSolution, CloudflareChallengeSolution, ReCaptchaSolution, TurnstileSolution,
};

// Re-export proxy types for convenience (also available at crate root)
pub use crate::proxy::{ProxyConfig, ProxyType};
//! # Capsolver Provider
//!
//! Implementation of the captcha solver provider for [Capsolver](https://capsolver.com).
//!
//! ## Supported Captcha Types
//!
//! | Type | Task Constructor | Proxy Required |
//! |------|-----------------|----------------|
//! | ReCaptcha V2 | `CapsolverTask::recaptcha_v2()` | No |
//! | ReCaptcha V2 Invisible | `CapsolverTask::recaptcha_v2_invisible()` | No |
//! | ReCaptcha V2 Enterprise | `CapsolverTask::recaptcha_v2_enterprise()` | No |
//! | ReCaptcha V2 Enterprise | `CapsolverTask::recaptcha_v2_enterprise_with_proxy()` | Yes |
//! | ReCaptcha V3 | `CapsolverTask::recaptcha_v3()` | No |
//! | ReCaptcha V3 | `CapsolverTask::recaptcha_v3_with_action()` | No |
//! | ReCaptcha V3 | `CapsolverTask::recaptcha_v3_with_proxy()` | Yes |
//! | ReCaptcha V3 Enterprise | `CapsolverTask::recaptcha_v3_enterprise()` | No |
//! | ReCaptcha V3 Enterprise | `CapsolverTask::recaptcha_v3_enterprise_with_proxy()` | Yes |
//! | Cloudflare Turnstile | `CapsolverTask::turnstile()` | No |
//! | Cloudflare Turnstile | `CapsolverTask::turnstile_with_metadata()` | No |
//! | Cloudflare Challenge | `CapsolverTask::cloudflare_challenge()` | Yes |
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use captcha_solvers::providers::capsolver::{CapsolverClient, CapsolverProvider, CapsolverTask};
//! use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait};
//! use std::time::Duration;
//!
//! // Create client with default API URL
//! let client = CapsolverClient::new("your_api_key")?;
//! let provider = CapsolverProvider::new(client);
//! let service = CaptchaSolverService::with_provider(provider);
//!
//! // Solve ReCaptcha V2
//! let task = CapsolverTask::recaptcha_v2("https://example.com", "site_key");
//! let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;
//! let token = solution.into_recaptcha().token();
//! ```
//!
//! ## Client Configuration
//!
//! The client can be configured using the builder pattern:
//!
//! ```rust,ignore
//! use captcha_solvers::providers::capsolver::CapsolverClient;
//! use url::Url;
//!
//! // Simple: default API URL
//! let client = CapsolverClient::new("api_key")?;
//!
//! // Custom URL
//! let client = CapsolverClient::with_url(
//!     Url::parse("https://api.capsolver.com")?,
//!     "api_key"
//! )?;
//!
//! // Full builder
//! let client = CapsolverClient::builder("api_key")
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

mod client;
mod errors;
mod provider;
mod response;
mod types;

#[cfg(test)]
mod tests;

// Client
pub use client::{CapsolverClient, CapsolverClientBuilder, DEFAULT_API_URL};

// Errors
pub use errors::{CapsolverApiError, CapsolverError, CapsolverErrorCode};

// Provider
pub use provider::CapsolverProvider;

// Tasks
pub use types::{CapsolverTask, TurnstileMetadata};

// Solutions
pub use types::{
    CapsolverSolution, CloudflareChallengeSolution, ReCaptchaSolution, TurnstileSolution,
};

// Re-export proxy types for convenience (also available at crate root)
pub use crate::proxy::{ProxyConfig, ProxyType};
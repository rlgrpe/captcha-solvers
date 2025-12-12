//! # Captcha Solvers
//!
//! A generic captcha solving library with provider abstraction and fluent builder pattern.
//!
//! This library provides a unified interface for working with different captcha
//! solving services. It supports multiple captcha types including ReCaptcha V2/V3,
//! Cloudflare Turnstile, and Cloudflare Challenge.
//!
//! ## Supported Providers
//!
//! | Provider | Feature | Website |
//! |----------|---------|---------|
//! | Capsolver | `capsolver` (default) | <https://capsolver.com> |
//! | RuCaptcha | `rucaptcha` | <https://rucaptcha.com> |
//!
//! ## Supported Captcha Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`ReCaptchaV2`] | Standard and Enterprise, visible and invisible |
//! | [`ReCaptchaV3`] | Score-based with action support |
//! | [`Turnstile`] | Cloudflare Turnstile widget |
//! | [`CloudflareChallenge`] | Full page challenge bypass (Capsolver only) |
//!
//! ## Quick Start
//!
//! Use the shared task types with any provider:
//!
//! ```rust,ignore
//! use captcha_solvers::{
//!     ReCaptchaV2, Turnstile, ProxyConfig,
//!     CaptchaSolverService, CaptchaSolverServiceTrait,
//!     providers::capsolver::CapsolverProvider,
//! };
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create provider with API key
//!     let provider = CapsolverProvider::new("your_api_key")?;
//!     let service = CaptchaSolverService::with_provider(provider);
//!
//!     // Use shared task types with builder pattern
//!     let task = ReCaptchaV2::new("https://example.com", "site_key")
//!         .invisible()
//!         .enterprise();
//!
//!     // Solve the captcha
//!     let solution = service
//!         .solve_captcha(task, Duration::from_secs(120))
//!         .await?;
//!     println!("Token: {}", solution.into_recaptcha().token());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Shared Task Types with Builder Pattern
//!
//! All task types use a fluent builder pattern with public fields:
//!
//! ```
//! use captcha_solvers::{ReCaptchaV2, ReCaptchaV3, Turnstile, ProxyConfig};
//!
//! // ReCaptcha V2 - chain builder methods
//! let task = ReCaptchaV2::new("https://example.com", "site_key")
//!     .invisible()
//!     .enterprise()
//!     .with_action("submit");
//!
//! // Direct field access
//! assert!(task.is_invisible);
//! assert!(task.is_enterprise);
//!
//! // ReCaptcha V3 - with score threshold
//! let task = ReCaptchaV3::new("https://example.com", "site_key")
//!     .with_action("login")
//!     .with_min_score(0.9);
//!
//! // Turnstile - with metadata
//! let task = Turnstile::new("https://example.com", "0x4AAAA...")
//!     .with_action("verify")
//!     .with_cdata("custom-data");
//!
//! // With proxy
//! let proxy = ProxyConfig::http("192.168.1.1", 8080);
//! let task = ReCaptchaV2::new("https://example.com", "site_key")
//!     .with_proxy(proxy);
//! ```
//!
//! ## Cloudflare Challenge (Capsolver only)
//!
//! ```rust,ignore
//! use captcha_solvers::{CloudflareChallenge, ProxyConfig};
//! use captcha_solvers::providers::capsolver::CapsolverProvider;
//!
//! // Cloudflare Challenge requires a static/sticky proxy
//! let proxy = ProxyConfig::http("192.168.1.1", 8080)
//!     .with_auth("user", "pass");
//!
//! let task = CloudflareChallenge::new("https://protected-site.com", proxy)
//!     .with_user_agent("Mozilla/5.0...");
//!
//! // Only supported by Capsolver
//! let provider = CapsolverProvider::new("api_key")?;
//! let service = CaptchaSolverService::with_provider(provider);
//! let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;
//! ```
//!
//! ## Provider Configuration
//!
//! ```rust,ignore
//! use captcha_solvers::providers::capsolver::CapsolverProvider;
//! use url::Url;
//!
//! // Simple: use default URL
//! let provider = CapsolverProvider::new("api_key")?;
//!
//! // Custom URL
//! let provider = CapsolverProvider::with_url(
//!     Url::parse("https://custom-api.example.com")?,
//!     "api_key"
//! )?;
//!
//! // Full builder for advanced configuration
//! let provider = CapsolverProvider::builder("api_key")
//!     .url(Url::parse("https://custom-api.example.com")?)
//!     .http_client(custom_http_client)
//!     .build()?;
//! ```
//!
//! ## Architecture
//!
//! ```text
//! Shared Task Types (ReCaptchaV2, Turnstile, etc.)
//!         │
//!         │  Into<CaptchaTask>
//!         ▼
//! CaptchaSolverService<P>
//!         │
//!         ▼
//! RetryableProvider<P>  (optional retry wrapper)
//!         │
//!         ▼
//!     Provider          (trait: CapsolverProvider, RucaptchaProvider)
//! ```
//!
//! ## Features
//!
//! - `capsolver` - Capsolver provider support (enabled by default)
//! - `rucaptcha` - RuCaptcha provider support
//! - `tracing` - OpenTelemetry tracing instrumentation (enabled by default)

pub mod errors;
pub mod provider;
pub mod providers;
pub mod proxy;
pub mod response;
pub mod retry;
pub mod serde_helpers;
pub mod service;
pub mod tasks;
pub mod types;

// Re-export commonly used types at the crate root
pub use errors::{RetryableError, UnsupportedTaskError};
pub use provider::{Provider, RetryableProvider};
pub use proxy::{
    CapsolverProxyFields, ProxyConfig, ProxyType, RucaptchaProxyFields,
    serialize_capsolver_proxy_type, serialize_rucaptcha_proxy_type,
};
pub use retry::RetryConfig;
pub use service::{
    CaptchaSolverService, CaptchaSolverServiceConfig, CaptchaSolverServiceTrait, ServiceError,
};
pub use tasks::{CaptchaTask, CloudflareChallenge, ReCaptchaV2, ReCaptchaV3, Turnstile};
pub use types::TaskId;
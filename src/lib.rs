//! # Captcha Solvers
//!
//! A generic captcha solving library with provider abstraction.
//!
//! This library provides a unified interface for working with different captcha
//! solving services. It supports multiple captcha types including ReCaptcha V2/V3,
//! Cloudflare Turnstile, and Cloudflare Challenge.
//!
//! ## Supported Providers
//!
//! - **Capsolver** - <https://capsolver.com> (feature: `capsolver`)
//! - **RuCaptcha** - <https://rucaptcha.com> (feature: `rucaptcha`)
//!
//! ## Supported Captcha Types
//!
//! - **ReCaptcha V2** - Standard and Enterprise, visible and invisible
//! - **ReCaptcha V3** - Standard and Enterprise with action support
//! - **Cloudflare Turnstile** - With optional metadata
//! - **Cloudflare Challenge** - Full page challenge bypass (Capsolver only)
//!
//! ## Architecture
//!
//! ```text
//! CaptchaSolverService<P>
//!         │
//!         ▼
//! RetryableProvider<P>  (optional retry wrapper)
//!         │
//!         ▼
//!     Provider          (trait implemented by each provider)
//!         │
//!         ▼
//!   Provider Client     (HTTP client for the specific service)
//! ```
//!
//! ## Features
//!
//! - `capsolver` - Capsolver provider support (enabled by default)
//! - `rucaptcha` - RuCaptcha provider support
//! - `tracing` - OpenTelemetry tracing instrumentation (enabled by default)
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use captcha_solvers::{
//!     CaptchaSolverService, CaptchaSolverServiceConfig, CaptchaSolverServiceTrait,
//!     providers::capsolver::{CapsolverClient, CapsolverProvider, CapsolverTask},
//! };
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client (uses default API URL)
//!     let client = CapsolverClient::new("your_api_key")?;
//!     let provider = CapsolverProvider::new(client);
//!     let service = CaptchaSolverService::with_provider(provider);
//!
//!     // Solve a Turnstile captcha
//!     let task = CapsolverTask::turnstile("https://example.com", "site_key");
//!     let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;
//!     println!("Token: {}", solution.into_turnstile().token());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Examples by Captcha Type
//!
//! ### ReCaptcha V2
//! ```rust,ignore
//! // Standard ReCaptcha V2
//! let task = CapsolverTask::recaptcha_v2("https://example.com", "site_key");
//!
//! // Invisible ReCaptcha V2
//! let task = CapsolverTask::recaptcha_v2_invisible("https://example.com", "site_key");
//!
//! // Enterprise with proxy
//! let task = CapsolverTask::recaptcha_v2_enterprise_with_proxy(
//!     "https://example.com",
//!     "site_key",
//!     "http://user:pass@proxy:8080"
//! );
//! ```
//!
//! ### ReCaptcha V3
//! ```rust,ignore
//! // Standard ReCaptcha V3
//! let task = CapsolverTask::recaptcha_v3("https://example.com", "site_key");
//!
//! // With action parameter
//! let task = CapsolverTask::recaptcha_v3_with_action(
//!     "https://example.com",
//!     "site_key",
//!     "submit"
//! );
//! ```
//!
//! ### Cloudflare Turnstile
//! ```rust,ignore
//! use captcha_solvers::providers::capsolver::TurnstileMetadata;
//!
//! // Simple Turnstile
//! let task = CapsolverTask::turnstile("https://example.com", "site_key");
//!
//! // With metadata
//! let metadata = TurnstileMetadata {
//!     action: Some("login".to_string()),
//!     cdata: None,
//! };
//! let task = CapsolverTask::turnstile_with_metadata("https://example.com", "site_key", metadata);
//! ```
//!
//! ### Cloudflare Challenge
//! ```rust,ignore
//! // Requires a static/sticky proxy (not rotating)
//! let task = CapsolverTask::cloudflare_challenge(
//!     "https://example.com",
//!     "http://user:pass@proxy:8080"
//! );
//! ```
//!
//! ## Client Configuration
//!
//! ```rust,ignore
//! use captcha_solvers::providers::capsolver::{CapsolverClient, DEFAULT_API_URL};
//! use url::Url;
//!
//! // Simple: use default URL
//! let client = CapsolverClient::new("api_key")?;
//!
//! // Custom URL
//! let client = CapsolverClient::with_url(
//!     Url::parse("https://custom-api.example.com")?,
//!     "api_key"
//! )?;
//!
//! // Full builder for advanced configuration
//! let client = CapsolverClient::builder("api_key")
//!     .url(Url::parse("https://custom-api.example.com")?)
//!     .http_client(custom_http_client)
//!     .build()?;
//! ```

pub mod client;
pub mod errors;
pub mod provider;
pub mod providers;
pub mod proxy;
pub mod response;
pub mod retry;
pub mod serde_helpers;
pub mod service;
pub mod types;

// Re-export commonly used types at the crate root
pub use errors::RetryableError;
pub use provider::{Provider, RetryableProvider};
pub use proxy::{
    CapsolverProxyFields, ProxyConfig, ProxyType, RucaptchaProxyFields,
    serialize_capsolver_proxy_type, serialize_rucaptcha_proxy_type,
};
pub use retry::RetryConfig;
pub use service::{
    CaptchaSolverService, CaptchaSolverServiceConfig, CaptchaSolverServiceTrait, ServiceError,
};
pub use types::TaskId;
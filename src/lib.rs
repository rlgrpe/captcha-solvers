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
//!     capsolver::CapsolverProvider,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create provider with API key
//!     let provider = CapsolverProvider::new("your_api_key")?;
//!     let service = CaptchaSolverService::new(provider);
//!
//!     // Use shared task types with builder pattern
//!     let task = ReCaptchaV2::new("https://example.com", "site_key")
//!         .invisible()
//!         .enterprise();
//!
//!     // Solve the captcha (uses default timeout from config)
//!     let solution = service.solve_captcha(task).await?;
//!     println!("Token: {}", solution.into_recaptcha().token());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Service Configuration
//!
//! Configure timeout and polling behavior using presets or custom values:
//!
//! ```rust,ignore
//! use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceConfig};
//! use std::time::Duration;
//!
//! // Using presets
//! let service = CaptchaSolverService::with_config(
//!     provider,
//!     CaptchaSolverServiceConfig::fast(),  // 60s timeout, 2s poll
//! );
//!
//! // Using builder pattern
//! let service = CaptchaSolverService::builder(provider)
//!     .timeout(Duration::from_secs(180))
//!     .poll_interval(Duration::from_secs(5))
//!     .build();
//! ```
//!
//! ### Available Presets
//!
//! | Preset | Timeout | Poll Interval | Use Case |
//! |--------|---------|---------------|----------|
//! | `fast()` | 60s | 2s | Development/testing |
//! | `balanced()` | 120s | 3s | Most production use (default) |
//! | `patient()` | 300s | 5s | Slow providers, complex captchas |
//!
//! ## Cancellation Support
//!
//! Long-running solve operations can be cancelled:
//!
//! ```rust,ignore
//! use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait, ReCaptchaV2};
//! use tokio_util::sync::CancellationToken;
//!
//! let cancel_token = CancellationToken::new();
//! let token_clone = cancel_token.clone();
//!
//! // Cancel after 30 seconds
//! tokio::spawn(async move {
//!     tokio::time::sleep(Duration::from_secs(30)).await;
//!     token_clone.cancel();
//! });
//!
//! let task = ReCaptchaV2::new("https://example.com", "site_key");
//! match service.solve_captcha_cancellable(task, cancel_token).await {
//!     Ok(solution) => println!("Got solution"),
//!     Err(e) if e.is_cancelled() => println!("Operation was cancelled"),
//!     Err(e) => println!("Error: {}", e),
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
//! ## Retry Support
//!
//! Wrap providers with automatic retry logic:
//!
//! ```rust,ignore
//! use captcha_solvers::{CaptchaRetryableProvider, RetryConfig};
//!
//! let provider = CapsolverProvider::new("api_key")?;
//!
//! // With default retry config
//! let retryable = CaptchaRetryableProvider::new(provider.clone());
//!
//! // With custom config and retry callback
//! let retryable = CaptchaRetryableProvider::with_config(
//!     provider,
//!     RetryConfig::default().with_max_retries(5),
//! )
//! .with_on_retry(|error, duration| {
//!     println!("Retrying after {:?} due to: {}", duration, error);
//! });
//!
//! let service = CaptchaSolverService::new(retryable);
//! ```
//!
//! ## Cloudflare Challenge (Capsolver only)
//!
//! ```rust,ignore
//! use captcha_solvers::{CloudflareChallenge, ProxyConfig};
//! use captcha_solvers::capsolver::CapsolverProvider;
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
//! let service = CaptchaSolverService::new(provider);
//! let solution = service.solve_captcha(task).await?;
//! ```
//!
//! ## Provider Configuration
//!
//! ```rust,ignore
//! use captcha_solvers::capsolver::CapsolverProvider;
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
//! CaptchaRetryableProvider<P>  (optional retry wrapper)
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
//! - `metrics` - OpenTelemetry metrics support

// Internal modules (hidden from users)
mod errors;
mod providers;
mod service;
mod solutions;
mod tasks;
pub(crate) mod utils;

// ============================================================================
// Provider Modules (feature-gated, expose provider-specific types)
// ============================================================================

#[cfg(feature = "capsolver")]
pub mod capsolver {
    //! Capsolver provider implementation.
    //!
    //! See [`CapsolverProvider`] for usage details.
    pub use crate::providers::capsolver::*;
}

#[cfg(feature = "rucaptcha")]
pub mod rucaptcha {
    //! RuCaptcha provider implementation.
    //!
    //! See [`RucaptchaProvider`] for usage details.
    pub use crate::providers::rucaptcha::*;
}

// ============================================================================
// Public API - Core Types
// ============================================================================

// Error handling
pub use errors::{RetryableError, UnsupportedTaskError};

// Provider abstraction
pub use providers::{CaptchaRetryableProvider, OnRetryCallback, Provider};

// Service
pub use service::{
    CaptchaSolverService, CaptchaSolverServiceBuilder, CaptchaSolverServiceConfig,
    CaptchaSolverServiceConfigBuilder, CaptchaSolverServiceTrait, ConfigError, MIN_POLL_INTERVAL,
    MIN_TIMEOUT, ServiceError,
};

// Re-export CancellationToken for convenience
pub use tokio_util::sync::CancellationToken;

// ============================================================================
// Public API - Task Types
// ============================================================================

pub use tasks::{CaptchaTask, CloudflareChallenge, ReCaptchaV2, ReCaptchaV3, Turnstile};

// ============================================================================
// Public API - Solution Types
// ============================================================================

pub use solutions::{
    CloudflareChallengeSolution, ProviderSolution, ReCaptchaSolution, TurnstileSolution,
};

// ============================================================================
// Public API - Utilities
// ============================================================================

pub use utils::proxy::{ProxyConfig, ProxyType};
pub use utils::retry::RetryConfig;
pub use utils::types::TaskId;

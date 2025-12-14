//! Captcha solver service module.
//!
//! This module provides the high-level service interface for solving captchas.
//! It handles task creation, polling, and error management.
//!
//! # Components
//!
//! - [`CaptchaSolverService`] - The main service struct
//! - [`CaptchaSolverServiceBuilder`] - Builder for service configuration
//! - [`CaptchaSolverServiceTrait`] - Trait for service implementations
//! - [`CaptchaSolverServiceConfig`] - Service configuration with presets
//! - [`ServiceError`] - Service-level errors
//! - [`ConfigError`] - Configuration validation errors
//!
//! # Example
//!
//! ```rust,ignore
//! use captcha_solvers::{
//!     CaptchaSolverService, CaptchaSolverServiceTrait,
//!     ReCaptchaV2, capsolver::CapsolverProvider,
//! };
//! use std::time::Duration;
//!
//! let provider = CapsolverProvider::new("api_key")?;
//! let service = CaptchaSolverService::new(provider);
//!
//! let task = ReCaptchaV2::new("https://example.com", "site_key");
//! let solution = service.solve_captcha(task).await?;
//! ```
//!
//! # Configuration Presets
//!
//! ```rust,ignore
//! use captcha_solvers::CaptchaSolverServiceConfig;
//!
//! // Fast preset - for development/testing
//! let fast = CaptchaSolverServiceConfig::fast();
//!
//! // Balanced preset (default)
//! let balanced = CaptchaSolverServiceConfig::balanced();
//!
//! // Patient preset - for slow providers
//! let patient = CaptchaSolverServiceConfig::patient();
//! ```
//!
//! # Builder Pattern
//!
//! ```rust,ignore
//! use captcha_solvers::CaptchaSolverService;
//! use std::time::Duration;
//!
//! let service = CaptchaSolverService::builder(provider)
//!     .timeout(Duration::from_secs(180))
//!     .poll_interval(Duration::from_secs(5))
//!     .build();
//! ```

mod config;
mod errors;
mod structure;
mod traits;

pub use config::{
    CaptchaSolverServiceConfig, CaptchaSolverServiceConfigBuilder, ConfigError, MIN_POLL_INTERVAL,
    MIN_TIMEOUT,
};
pub use errors::ServiceError;
pub use structure::{CaptchaSolverService, CaptchaSolverServiceBuilder};
pub use traits::CaptchaSolverServiceTrait;

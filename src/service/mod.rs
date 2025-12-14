//! Captcha solver service module.
//!
//! This module provides the high-level service interface for solving captchas.
//! It handles task creation, polling, and error management.
//!
//! # Components
//!
//! - [`CaptchaSolverService`] - The main service struct
//! - [`CaptchaSolverServiceTrait`] - Trait for service implementations
//! - [`CaptchaSolverServiceConfig`] - Service configuration
//! - [`ServiceError`] - Service-level errors
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
//! let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;
//! ```

mod config;
mod errors;
mod structure;
mod traits;

pub use config::CaptchaSolverServiceConfig;
pub use errors::ServiceError;
pub use structure::CaptchaSolverService;
pub use traits::CaptchaSolverServiceTrait;

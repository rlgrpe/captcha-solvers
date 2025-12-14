//! Captcha solver provider implementations.
//!
//! This module contains the core [`Provider`] trait and provider implementations.

mod retryable;
pub(crate) mod traits;

pub use retryable::{CaptchaRetryableProvider, OnRetryCallback};
pub use traits::Provider;

#[cfg(feature = "capsolver")]
pub mod capsolver;

#[cfg(feature = "rucaptcha")]
pub mod rucaptcha;

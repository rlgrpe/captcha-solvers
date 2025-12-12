//! Provider-agnostic captcha task types with builder pattern.
//!
//! This module provides shared task definitions that work with any captcha solving
//! provider. Tasks are created using a fluent builder pattern.
//!
//! # Supported Task Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`ReCaptchaV2`] | Google reCAPTCHA V2 (checkbox or invisible) |
//! | [`ReCaptchaV3`] | Google reCAPTCHA V3 (score-based) |
//! | [`Turnstile`] | Cloudflare Turnstile widget |
//! | [`CloudflareChallenge`] | Full-page Cloudflare challenge bypass |
//!
//! # Usage
//!
//! ```
//! use captcha_solvers::tasks::{ReCaptchaV2, ReCaptchaV3, Turnstile};
//!
//! // ReCaptcha V2 - invisible enterprise with proxy
//! let task = ReCaptchaV2::new("https://example.com", "site-key")
//!     .invisible()
//!     .enterprise();
//!
//! // ReCaptcha V3 - with action and minimum score
//! let task = ReCaptchaV3::new("https://example.com", "site-key")
//!     .with_action("submit")
//!     .with_min_score(0.9);
//!
//! // Turnstile - with metadata
//! let task = Turnstile::new("https://example.com", "0x4AAAA...")
//!     .with_action("login");
//! ```
//!
//! # Unified Task Type
//!
//! The [`CaptchaTask`] enum provides a unified type for all task types:
//!
//! ```
//! use captcha_solvers::tasks::{CaptchaTask, ReCaptchaV2, Turnstile};
//!
//! // Individual task types convert to CaptchaTask automatically
//! let task: CaptchaTask = ReCaptchaV2::new("https://example.com", "site-key")
//!     .enterprise()
//!     .into();
//!
//! let task: CaptchaTask = Turnstile::new("https://example.com", "0x4AAAA...")
//!     .into();
//! ```
//!
//! # Direct Field Access
//!
//! All task fields are public and can be accessed directly:
//!
//! ```
//! use captcha_solvers::tasks::ReCaptchaV2;
//!
//! let task = ReCaptchaV2::new("https://example.com", "site-key")
//!     .invisible()
//!     .enterprise();
//!
//! // Direct field access
//! assert_eq!(task.website_url, "https://example.com");
//! assert_eq!(task.website_key, "site-key");
//! assert!(task.is_invisible);
//! assert!(task.is_enterprise);
//!
//! // Or use getter methods
//! assert_eq!(task.website_url(), "https://example.com");
//! assert!(task.is_invisible());
//! ```

mod cloudflare;
mod recaptcha;

pub use cloudflare::{CloudflareChallenge, Turnstile};
pub use recaptcha::{ReCaptchaV2, ReCaptchaV3};

use std::fmt;

/// Unified captcha task type that can represent any supported captcha.
///
/// This enum wraps all individual task types and is used by providers
/// to accept any captcha task uniformly.
///
/// # Example
///
/// ```
/// use captcha_solvers::tasks::{CaptchaTask, ReCaptchaV2, Turnstile};
///
/// // Create from individual task types
/// let task: CaptchaTask = ReCaptchaV2::new("https://example.com", "site-key")
///     .enterprise()
///     .into();
///
/// // Or use the From implementations
/// let recaptcha = ReCaptchaV2::new("https://example.com", "site-key");
/// let task = CaptchaTask::from(recaptcha);
/// ```
#[derive(Debug, Clone)]
pub enum CaptchaTask {
    /// Google reCAPTCHA V2
    ReCaptchaV2(ReCaptchaV2),
    /// Google reCAPTCHA V3
    ReCaptchaV3(ReCaptchaV3),
    /// Cloudflare Turnstile
    Turnstile(Turnstile),
    /// Cloudflare Challenge (full page bypass)
    CloudflareChallenge(CloudflareChallenge),
}

impl fmt::Display for CaptchaTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReCaptchaV2(task) => {
                let variant = match (task.is_invisible, task.is_enterprise) {
                    (true, true) => "ReCaptchaV2InvisibleEnterprise",
                    (true, false) => "ReCaptchaV2Invisible",
                    (false, true) => "ReCaptchaV2Enterprise",
                    (false, false) => "ReCaptchaV2",
                };
                write!(f, "{}", variant)
            }
            Self::ReCaptchaV3(task) => {
                if task.is_enterprise {
                    write!(f, "ReCaptchaV3Enterprise")
                } else {
                    write!(f, "ReCaptchaV3")
                }
            }
            Self::Turnstile(_) => write!(f, "Turnstile"),
            Self::CloudflareChallenge(_) => write!(f, "CloudflareChallenge"),
        }
    }
}

impl From<ReCaptchaV2> for CaptchaTask {
    fn from(task: ReCaptchaV2) -> Self {
        Self::ReCaptchaV2(task)
    }
}

impl From<ReCaptchaV3> for CaptchaTask {
    fn from(task: ReCaptchaV3) -> Self {
        Self::ReCaptchaV3(task)
    }
}

impl From<Turnstile> for CaptchaTask {
    fn from(task: Turnstile) -> Self {
        Self::Turnstile(task)
    }
}

impl From<CloudflareChallenge> for CaptchaTask {
    fn from(task: CloudflareChallenge) -> Self {
        Self::CloudflareChallenge(task)
    }
}

//! Service configuration types.

use std::time::Duration;

/// Configuration for the captcha solver service.
///
/// Controls behavior like polling intervals when waiting for captcha solutions.
///
/// # Example
///
/// ```rust
/// use captcha_solvers::CaptchaSolverServiceConfig;
/// use std::time::Duration;
///
/// // Use defaults (3 second poll interval)
/// let config = CaptchaSolverServiceConfig::default();
///
/// // Custom poll interval
/// let config = CaptchaSolverServiceConfig {
///     poll_interval: Duration::from_secs(5),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct CaptchaSolverServiceConfig {
    /// Interval between polling attempts when waiting for solution.
    ///
    /// Default: 3 seconds
    pub poll_interval: Duration,
}

impl Default for CaptchaSolverServiceConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(3),
        }
    }
}

impl CaptchaSolverServiceConfig {
    /// Create a new configuration with the specified poll interval.
    pub fn with_poll_interval(poll_interval: Duration) -> Self {
        Self { poll_interval }
    }
}

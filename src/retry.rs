use backon::{BackoffBuilder, ExponentialBuilder};
use std::time::Duration;

/// Default retry strategy for captcha solver operations
///
/// Uses exponential backoff with:
/// - Initial delay: 1 second
/// - Max delay: 30 seconds
/// - Factor: 2x
/// - Max retries: 3
pub fn default_retry_strategy() -> impl BackoffBuilder {
    ExponentialBuilder::default()
        .with_min_delay(Duration::from_secs(1))
        .with_max_delay(Duration::from_secs(30))
        .with_factor(2.0)
        .with_max_times(3)
}

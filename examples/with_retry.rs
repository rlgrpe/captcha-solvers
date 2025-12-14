//! Example with automatic retry on transient failures.
//!
//! Run with: `cargo run --example with_retry`
//!
//! Required environment variable:
//! - `CAPSOLVER_API_KEY` - Your Capsolver API key

use captcha_solvers::capsolver::CapsolverProvider;
use captcha_solvers::{
    CaptchaRetryableProvider, CaptchaSolverService, CaptchaSolverServiceTrait, ReCaptchaV2,
    RetryConfig,
};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("CAPSOLVER_API_KEY").expect("CAPSOLVER_API_KEY must be set");

    // Create base provider
    let base_provider = CapsolverProvider::new(api_key)?;

    // Configure retry behavior with exponential backoff
    let retry_config = RetryConfig::default()
        .with_max_retries(5)
        .with_min_delay(Duration::from_millis(500))
        .with_max_delay(Duration::from_secs(30))
        .with_factor(2.0);

    // Wrap provider with retry logic and add a callback for retry notifications
    let provider = CaptchaRetryableProvider::with_config(base_provider, retry_config)
        .with_on_retry(|error, duration| {
            eprintln!(
                "Retry triggered: will retry after {:?} due to error: {}",
                duration, error
            );
        });

    let service = CaptchaSolverService::new(provider);

    let task = ReCaptchaV2::new(
        "https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high",
        "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd",
    );

    println!("Solving with automatic retry on transient failures...");

    let solution = service.solve_captcha(task).await?;

    let recaptcha = solution.into_recaptcha();
    println!("Solved! Token length: {}", recaptcha.token().len());

    Ok(())
}

//! Example showing service configuration options.
//!
//! Run with: `cargo run --example with_config`
//!
//! Required environment variable:
//! - `CAPSOLVER_API_KEY` - Your Capsolver API key

use captcha_solvers::capsolver::CapsolverProvider;
use captcha_solvers::{
    CaptchaSolverService, CaptchaSolverServiceConfig, CaptchaSolverServiceTrait, ReCaptchaV2,
};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("CAPSOLVER_API_KEY").expect("CAPSOLVER_API_KEY must be set");
    let provider = CapsolverProvider::new(api_key)?;

    // === Configuration Presets ===

    // Fast preset: 60s timeout, 2s poll interval (good for development)
    let _fast_config = CaptchaSolverServiceConfig::fast();

    // Balanced preset: 120s timeout, 3s poll interval (default)
    let _balanced_config = CaptchaSolverServiceConfig::balanced();

    // Patient preset: 300s timeout, 5s poll interval (for slow providers)
    let _patient_config = CaptchaSolverServiceConfig::patient();

    // === Custom Configuration with Builder ===

    // Method 1: Using the service builder
    let service = CaptchaSolverService::builder(provider.clone())
        .timeout(Duration::from_secs(90))
        .poll_interval(Duration::from_secs(4))
        .build();

    println!(
        "Service config: timeout={:?}, poll_interval={:?}",
        service.config().timeout,
        service.config().poll_interval
    );

    // Method 2: Using config builder directly
    let config = CaptchaSolverServiceConfig::builder()
        .timeout(Duration::from_secs(180))
        .poll_interval(Duration::from_secs(5))
        .build();

    let _service = CaptchaSolverService::with_config(provider.clone(), config);

    // Method 3: Config with validation
    let config_result = CaptchaSolverServiceConfig::builder()
        .timeout(Duration::from_secs(60))
        .poll_interval(Duration::from_secs(2))
        .try_build(); // Returns Result with validation

    match config_result {
        Ok(config) => {
            println!("Valid config: timeout={:?}", config.timeout);
        }
        Err(e) => {
            eprintln!("Invalid config: {}", e);
        }
    }

    // Method 4: Modify existing config with fluent methods
    let config = CaptchaSolverServiceConfig::default()
        .with_timeout(Duration::from_secs(150))
        .with_poll_interval(Duration::from_secs(3));

    let service = CaptchaSolverService::with_config(provider, config);

    // === Solve a captcha ===

    let task = ReCaptchaV2::new(
        "https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high",
        "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd",
    );

    println!("Solving with custom config...");

    let solution = service.solve_captcha(task).await?;

    let recaptcha = solution.into_recaptcha();
    println!("Solved! Token length: {}", recaptcha.token().len());

    Ok(())
}

//! Basic ReCaptcha V2 solving example.
//!
//! Run with: `cargo run --example basic_recaptcha_v2`
//!
//! Required environment variable:
//! - `CAPSOLVER_API_KEY` - Your Capsolver API key

use captcha_solvers::capsolver::CapsolverProvider;
use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait, ReCaptchaV2};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment
    let api_key = env::var("CAPSOLVER_API_KEY").expect("CAPSOLVER_API_KEY must be set");

    // Create provider and service
    let provider = CapsolverProvider::new(api_key)?;
    let service = CaptchaSolverService::new(provider);

    // Create a ReCaptcha V2 task
    // Demo site from https://2captcha.com/demo
    let task = ReCaptchaV2::new(
        "https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high",
        "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd",
    );

    println!("Solving ReCaptcha V2...");

    // Solve with 2 minute timeout
    let solution = service.solve_captcha(task).await?;

    // Extract the token
    let recaptcha = solution.into_recaptcha();
    println!("Solved! Token: {}...", &recaptcha.token()[..50]);

    Ok(())
}

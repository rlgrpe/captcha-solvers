//! Example using RuCaptcha provider instead of Capsolver.
//!
//! Run with: `cargo run --example rucaptcha_provider`
//!
//! Required environment variable:
//! - `RUCAPTCHA_API_KEY` - Your RuCaptcha API key

use captcha_solvers::providers::rucaptcha::RucaptchaProvider;
use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait, ReCaptchaV2};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("RUCAPTCHA_API_KEY").expect("RUCAPTCHA_API_KEY must be set");

    // RuCaptcha provider - same interface as Capsolver
    let provider = RucaptchaProvider::new(api_key)?;
    let service = CaptchaSolverService::with_provider(provider);

    let task = ReCaptchaV2::new(
        "https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high",
        "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd",
    );

    println!("Solving ReCaptcha V2 with RuCaptcha...");

    let solution = service
        .solve_captcha(task, Duration::from_secs(120))
        .await?;

    let recaptcha = solution.into_recaptcha();
    println!("Solved! Token length: {}", recaptcha.token().len());

    Ok(())
}

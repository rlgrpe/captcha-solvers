//! ReCaptcha V3 solving example with action parameter.
//!
//! Run with: `cargo run --example recaptcha_v3`
//!
//! Required environment variable:
//! - `CAPSOLVER_API_KEY` - Your Capsolver API key

use captcha_solvers::capsolver::CapsolverProvider;
use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait, ReCaptchaV3};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("CAPSOLVER_API_KEY").expect("CAPSOLVER_API_KEY must be set");

    let provider = CapsolverProvider::new(api_key)?;
    let service = CaptchaSolverService::new(provider);

    // ReCaptcha V3 with action and minimum score
    let task = ReCaptchaV3::new(
        "https://lessons.zennolab.com/captchas/recaptcha/v3.php?level=beta",
        "6Le0xVgUAAAAAIt20XEB4rVhYOODgTl00d8juDob",
    )
    .with_action("myverify")
    .with_min_score(0.9);

    println!("Solving ReCaptcha V3 with action 'myverify'...");

    let solution = service
        .solve_captcha(task, Duration::from_secs(120))
        .await?;

    let recaptcha = solution.into_recaptcha();
    println!("Solved! Token length: {}", recaptcha.token().len());

    Ok(())
}

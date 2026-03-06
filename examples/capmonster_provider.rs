//! Example using CapMonster Cloud provider.
//!
//! Run with: `cargo run --example capmonster_provider`
//!
//! Required environment variable:
//! - `CAPMONSTER_API_KEY` - Your CapMonster Cloud API key

use captcha_solvers::capmonster::CapmonsterProvider;
use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait, ReCaptchaV2};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("CAPMONSTER_API_KEY").expect("CAPMONSTER_API_KEY must be set");

    // CapMonster Cloud provider - same interface as Capsolver
    let provider = CapmonsterProvider::new(api_key)?;
    let service = CaptchaSolverService::new(provider);

    let task = ReCaptchaV2::new(
        "https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high",
        "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd",
    );

    println!("Solving ReCaptcha V2 with CapMonster Cloud...");

    let solution = service.solve_captcha(task).await?;

    let recaptcha = solution.into_recaptcha();
    println!("Solved! Token length: {}", recaptcha.token().len());

    Ok(())
}

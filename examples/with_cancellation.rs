//! Example showing how to cancel a long-running solve operation.
//!
//! Run with: `cargo run --example with_cancellation`
//!
//! Required environment variable:
//! - `CAPSOLVER_API_KEY` - Your Capsolver API key

use captcha_solvers::capsolver::CapsolverProvider;
use captcha_solvers::{
    CancellationToken, CaptchaSolverService, CaptchaSolverServiceConfig, CaptchaSolverServiceTrait,
    ReCaptchaV2,
};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("CAPSOLVER_API_KEY").expect("CAPSOLVER_API_KEY must be set");
    let provider = CapsolverProvider::new(api_key)?;

    // Use patient config with long timeout
    let service =
        CaptchaSolverService::with_config(provider, CaptchaSolverServiceConfig::patient());

    let task = ReCaptchaV2::new(
        "https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high",
        "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd",
    );

    // Create a cancellation token
    let cancel_token = CancellationToken::new();
    let token_clone = cancel_token.clone();

    // Spawn a task that will cancel after 10 seconds
    // (In a real application, this might be triggered by user input)
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("Cancelling the solve operation...");
        token_clone.cancel();
    });

    println!("Starting solve with cancellation support...");
    println!("(Will be cancelled after 10 seconds if not completed)");

    // Use the cancellable version
    match service.solve_captcha_cancellable(task, cancel_token).await {
        Ok(solution) => {
            let recaptcha = solution.into_recaptcha();
            println!(
                "Solved before cancellation! Token length: {}",
                recaptcha.token().len()
            );
        }
        Err(e) if e.is_cancelled() => {
            println!("Operation was cancelled!");
            if let Some(elapsed) = e.elapsed() {
                println!("  Elapsed time: {:?}", elapsed);
            }
            if let Some(polls) = e.poll_count() {
                println!("  Poll attempts: {}", polls);
            }
            if let Some(task_id) = e.task_id() {
                println!("  Task ID: {}", task_id);
            }
        }
        Err(e) if e.is_timeout() => {
            println!("Operation timed out!");
            if let Some(elapsed) = e.elapsed() {
                println!("  Elapsed time: {:?}", elapsed);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}

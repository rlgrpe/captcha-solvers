//! Cloudflare Turnstile solving example.
//!
//! Run with: `cargo run --example turnstile`
//!
//! Required environment variable:
//! - `CAPSOLVER_API_KEY` - Your Capsolver API key

use captcha_solvers::providers::capsolver::CapsolverProvider;
use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait, Turnstile};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("CAPSOLVER_API_KEY").expect("CAPSOLVER_API_KEY must be set");

    let provider = CapsolverProvider::new(api_key)?;
    let service = CaptchaSolverService::with_provider(provider);

    // Cloudflare Turnstile task
    let task = Turnstile::new(
        "https://visa.vfsglobal.com/uzb/ru/ltu/login",
        "0x4AAAAAABhlz7Ei4byodYjs",
    );

    println!("Solving Cloudflare Turnstile...");

    let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;

    let turnstile = solution.into_turnstile();
    println!("Solved! Token: {}...", &turnstile.token()[..50]);

    Ok(())
}

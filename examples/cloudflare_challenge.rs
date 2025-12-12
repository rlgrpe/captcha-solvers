//! Cloudflare Challenge solving example (requires proxy).
//!
//! Run with: `cargo run --example cloudflare_challenge`
//!
//! Required environment variables:
//! - `CAPSOLVER_API_KEY` - Your Capsolver API key
//! - `PROXY_HOST` - Proxy hostname or IP
//! - `PROXY_PORT` - Proxy port
//!
//! Optional environment variables:
//! - `PROXY_USER` - Proxy username
//! - `PROXY_PASSWORD` - Proxy password

use captcha_solvers::providers::capsolver::CapsolverProvider;
use captcha_solvers::{
    CaptchaSolverService, CaptchaSolverServiceTrait, CloudflareChallenge, ProxyConfig,
};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("CAPSOLVER_API_KEY").expect("CAPSOLVER_API_KEY must be set");
    let proxy_host = env::var("PROXY_HOST").expect("PROXY_HOST must be set");
    let proxy_port: u16 = env::var("PROXY_PORT")
        .expect("PROXY_PORT must be set")
        .parse()
        .expect("PROXY_PORT must be a valid port number");

    let provider = CapsolverProvider::new(api_key)?;
    let service = CaptchaSolverService::with_provider(provider);

    // Create proxy config (Cloudflare Challenge requires a proxy)
    let mut proxy = ProxyConfig::http(&proxy_host, proxy_port);

    // Add auth if provided
    if let (Ok(user), Ok(pass)) = (env::var("PROXY_USER"), env::var("PROXY_PASSWORD")) {
        proxy = proxy.with_auth(user, pass);
    }

    println!("Using proxy: {}:{}", proxy_host, proxy_port);

    // Cloudflare Challenge requires proxy - it solves the full-page challenge
    let task = CloudflareChallenge::new("https://www.moneysupermarket.com", proxy);

    println!("Solving Cloudflare Challenge (this may take a few minutes)...");

    let solution = service.solve_captcha(task, Duration::from_secs(180)).await?;

    let cf_solution = solution.into_cloudflare_challenge();
    println!("Solved!");
    println!("Token: {}...", &cf_solution.token()[..50.min(cf_solution.token().len())]);

    if let Some(clearance) = cf_solution.cf_clearance() {
        println!("cf_clearance cookie: {}", clearance);
    }

    Ok(())
}

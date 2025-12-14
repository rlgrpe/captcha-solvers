//! Example of solving captchas with a proxy.
//!
//! Run with: `cargo run --example with_proxy`
//!
//! Required environment variables:
//! - `CAPSOLVER_API_KEY` - Your Capsolver API key
//! - `PROXY_HOST` - Proxy hostname or IP
//! - `PROXY_PORT` - Proxy port

use captcha_solvers::capsolver::CapsolverProvider;
use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait, ProxyConfig, ReCaptchaV2};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("CAPSOLVER_API_KEY").expect("CAPSOLVER_API_KEY must be set");
    let proxy_host = env::var("PROXY_HOST").expect("PROXY_HOST must be set");
    let proxy_port: u16 = env::var("PROXY_PORT")
        .expect("PROXY_PORT must be set")
        .parse()?;

    let provider = CapsolverProvider::new(api_key)?;
    let service = CaptchaSolverService::new(provider);

    // Different proxy types available:
    // - ProxyConfig::http(host, port)
    // - ProxyConfig::https(host, port)
    // - ProxyConfig::socks4(host, port)
    // - ProxyConfig::socks5(host, port)
    let proxy = ProxyConfig::http(&proxy_host, proxy_port);

    // With authentication:
    // let proxy = ProxyConfig::socks5(&proxy_host, proxy_port)
    //     .with_auth("username", "password");

    let task = ReCaptchaV2::new(
        "https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high",
        "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd",
    )
    .with_proxy(proxy);

    println!(
        "Solving ReCaptcha V2 with proxy {}:{}...",
        proxy_host, proxy_port
    );

    let solution = service.solve_captcha(task).await?;

    let recaptcha = solution.into_recaptcha();
    println!("Solved! Token length: {}", recaptcha.token().len());

    Ok(())
}

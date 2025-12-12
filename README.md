# captcha-solvers

A generic Rust library for solving captchas through various provider services.

> **Disclaimer**: This library is provided as-is. I am not obligated to maintain it, fix bugs, or add features. If you want to contribute improvements, please submit a pull request.

## Features

- Provider-agnostic design with unified task types
- Fluent builder pattern for ergonomic API
- Automatic retry with exponential backoff
- Proxy support (HTTP, HTTPS, SOCKS4, SOCKS5)
- OpenTelemetry tracing (optional)

## Supported Providers

| Provider | Feature Flag | Cloudflare Challenge |
|----------|--------------|----------------------|
| [Capsolver](https://capsolver.com) | `capsolver` (default) | Yes |
| [RuCaptcha](https://rucaptcha.com) | `rucaptcha` (default) | No |

## Supported Captcha Types

- ReCaptcha V2 (standard, invisible, enterprise)
- ReCaptcha V3 (standard, enterprise)
- Cloudflare Turnstile
- Cloudflare Challenge (Capsolver only, requires proxy)

## Installation

```toml
[dependencies]
captcha-solvers = { git = "https://github.com/rlgrpe/captcha-solvers.git", tag = "v.0.1.0" }
```

To use only specific providers:

```toml
[dependencies]
captcha-solvers = { git = "https://github.com/rlgrpe/captcha-solvers.git", tag = "v.0.1.0", default-features = false, features = ["capsolver"] }
```

## Quick Start

```rust
use captcha_solvers::providers::capsolver::CapsolverProvider;
use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait, ReCaptchaV2};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = CapsolverProvider::new("your_api_key")?;
    let service = CaptchaSolverService::with_provider(provider);

    let task = ReCaptchaV2::new("https://example.com", "site_key");
    let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;

    let token = solution.into_recaptcha().token();
    println!("Token: {}", token);

    Ok(())
}
```

## Examples

### ReCaptcha V2

```rust
// Standard
let task = ReCaptchaV2::new("https://example.com", "site_key");

// Invisible
let task = ReCaptchaV2::new("https://example.com", "site_key").invisible();

// Enterprise
let task = ReCaptchaV2::new("https://example.com", "site_key").enterprise();

// With proxy
let proxy = ProxyConfig::http("192.168.1.1", 8080);
let task = ReCaptchaV2::new("https://example.com", "site_key").with_proxy(proxy);
```

### ReCaptcha V3

```rust
let task = ReCaptchaV3::new("https://example.com", "site_key")
    .with_action("submit")
    .with_min_score(0.9);
```

### Cloudflare Turnstile

```rust
let task = Turnstile::new("https://example.com", "site_key")
    .with_action("login");
```

### Cloudflare Challenge

```rust
// Requires proxy - must use Capsolver
let proxy = ProxyConfig::http("192.168.1.1", 8080).with_auth("user", "pass");
let task = CloudflareChallenge::new("https://protected-site.com", proxy);

let solution = service.solve_captcha(task, Duration::from_secs(180)).await?;
let cf_solution = solution.into_cloudflare_challenge();

println!("Token: {}", cf_solution.token());
if let Some(clearance) = cf_solution.cf_clearance() {
    println!("cf_clearance: {}", clearance);
}
```

### Using Proxy

```rust
// HTTP
let proxy = ProxyConfig::http("host", 8080);

// SOCKS5 with authentication
let proxy = ProxyConfig::socks5("host", 1080).with_auth("user", "pass");

// Available types: http, https, socks4, socks5
```

### Retry Configuration

```rust
use captcha_solvers::{RetryConfig, RetryableProvider};

let base_provider = CapsolverProvider::new("api_key")?;

let retry_config = RetryConfig::default()
    .with_max_retries(5)
    .with_min_delay(Duration::from_millis(500))
    .with_max_delay(Duration::from_secs(30));

let provider = RetryableProvider::with_config(base_provider, retry_config);
let service = CaptchaSolverService::with_provider(provider);
```

## Running Examples

Set your API key:

```bash
export CAPSOLVER_API_KEY=your_key_here
```

Run an example:

```bash
cargo run --example basic_recaptcha_v2
cargo run --example recaptcha_v3
cargo run --example turnstile
cargo run --example with_proxy
cargo run --example with_retry
cargo run --example rucaptcha_provider
```

For Cloudflare Challenge (requires proxy):

```bash
export PROXY_HOST=192.168.1.1
export PROXY_PORT=8080
cargo run --example cloudflare_challenge
```

## Contributing

This project is not actively maintained. If you'd like to add features or fix bugs, please submit a pull request. I'll review and merge contributions when I have time, but make no guarantees about response times.

## License

MIT

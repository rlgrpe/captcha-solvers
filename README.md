# captcha-solvers

A generic Rust library for solving captchas through various provider services.

> **[Sign up for CapSolver](https://dashboard.capsolver.com/passport/register?inviteCode=zhvlp56mC7mg)**

> **[Sign up for RuCaptcha](https://rucaptcha.com/?from=13331351)**

> **Disclaimer**: This library is provided as-is. I am not obligated to maintain it, fix bugs, or add features. If you want to contribute improvements, please submit a pull request.

## Features

- Provider-agnostic design with unified task types
- Fluent builder pattern for ergonomic API
- Service configuration with presets (fast, balanced, patient)
- Automatic retry with exponential backoff and callbacks
- Cancellation support for long-running operations
- Proxy support (HTTP, HTTPS, SOCKS4, SOCKS5)
- OpenTelemetry tracing (optional, `tracing` feature)
- OpenTelemetry metrics (optional, `metrics` feature)

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
- Image to Text (OCR recognition)

## Installation

```toml
[dependencies]
captcha-solvers = { git = "https://github.com/rlgrpe/captcha-solvers.git", tag = "v0.1.1" }
```

To use only specific providers:

```toml
[dependencies]
captcha-solvers = { git = "https://github.com/rlgrpe/captcha-solvers.git", tag = "v0.1.1", default-features = false, features = ["capsolver"] }
```

With metrics support:

```toml
[dependencies]
captcha-solvers = { git = "https://github.com/rlgrpe/captcha-solvers.git", tag = "v0.1.1", features = ["metrics"] }
```

## Quick Start

```rust
use captcha_solvers::capsolver::CapsolverProvider;
use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceTrait, ReCaptchaV2};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = CapsolverProvider::new("your_api_key")?;
    let service = CaptchaSolverService::new(provider);

    let task = ReCaptchaV2::new("https://example.com", "site_key");
    let solution = service.solve_captcha(task).await?;

    let token = solution.into_recaptcha().token();
    println!("Token: {}", token);

    Ok(())
}
```

## Service Configuration

### Configuration Presets

```rust
use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceConfig};
use captcha_solvers::capsolver::CapsolverProvider;

let provider = CapsolverProvider::new("api_key") ?;

// Fast preset: 60s timeout, 2s poll interval (good for development)
let service = CaptchaSolverService::with_config(provider, CaptchaSolverServiceConfig::fast());

// Balanced preset: 120s timeout, 3s poll interval (default)
let service = CaptchaSolverService::with_config(provider, CaptchaSolverServiceConfig::balanced());

// Patient preset: 300s timeout, 5s poll interval (for slow providers)
let service = CaptchaSolverService::with_config(provider, CaptchaSolverServiceConfig::patient());
```

### Custom Configuration with Builder

```rust
use captcha_solvers::{CaptchaSolverService, CaptchaSolverServiceConfig};
use std::time::Duration;

// Method 1: Using the service builder
let service = CaptchaSolverService::builder(provider)
.timeout(Duration::from_secs(90))
.poll_interval(Duration::from_secs(4))
.build();

// Method 2: Using config builder directly
let config = CaptchaSolverServiceConfig::builder()
.timeout(Duration::from_secs(180))
.poll_interval(Duration::from_secs(5))
.build();

let service = CaptchaSolverService::with_config(provider, config);

// Method 3: Config with validation
let config_result = CaptchaSolverServiceConfig::builder()
.timeout(Duration::from_secs(60))
.poll_interval(Duration::from_secs(2))
.try_build(); // Returns Result with validation

// Method 4: Modify existing config with fluent methods
let config = CaptchaSolverServiceConfig::default ()
.with_timeout(Duration::from_secs(150))
.with_poll_interval(Duration::from_secs(3));
```

## Cancellation Support

Cancel long-running solve operations using `CancellationToken`:

```rust
use captcha_solvers::{
    CancellationToken, CaptchaSolverService, CaptchaSolverServiceConfig,
    CaptchaSolverServiceTrait, ReCaptchaV2,
};
use std::time::Duration;

let service = CaptchaSolverService::with_config(provider, CaptchaSolverServiceConfig::patient());

let task = ReCaptchaV2::new("https://example.com", "site_key");

// Create a cancellation token
let cancel_token = CancellationToken::new();
let token_clone = cancel_token.clone();

// Spawn a task that will cancel after 10 seconds
tokio::spawn( async move {
tokio::time::sleep(Duration::from_secs(10)).await;
token_clone.cancel();
});

// Use the cancellable version
match service.solve_captcha_cancellable(task, cancel_token).await {
Ok(solution) => {
println ! ("Solved! Token: {}", solution.into_recaptcha().token());
}
Err(e) if e.is_cancelled() => {
println ! ("Operation was cancelled!");
if let Some(elapsed) = e.elapsed() {
println ! ("  Elapsed time: {:?}", elapsed);
}
if let Some(polls) = e.poll_count() {
println ! ("  Poll attempts: {}", polls);
}
}
Err(e) if e.is_timeout() => {
println ! ("Operation timed out!");
}
Err(e) => {
eprintln ! ("Error: {}", e);
}
}
```

## Examples

### ReCaptcha V2

```rust
use captcha_solvers::{ReCaptchaV2, ProxyConfig};

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
use captcha_solvers::ReCaptchaV3;

let task = ReCaptchaV3::new("https://example.com", "site_key")
    .with_action("submit")
    .with_min_score(0.9);
```

### Cloudflare Turnstile

```rust
use captcha_solvers::Turnstile;

let task = Turnstile::new("https://example.com", "site_key")
    .with_action("login");
```

### Cloudflare Challenge

```rust
use captcha_solvers::{CloudflareChallenge, ProxyConfig};

// Requires proxy - must use Capsolver
let proxy = ProxyConfig::http("192.168.1.1", 8080).with_auth("user", "pass");
let task = CloudflareChallenge::new("https://protected-site.com", proxy);

let solution = service.solve_captcha(task).await?;
let cf_solution = solution.into_cloudflare_challenge();

println!("Token: {}", cf_solution.token());
if let Some(clearance) = cf_solution.cf_clearance() {
    println!("cf_clearance: {}", clearance);
}
```

### Image to Text (OCR)

```rust
use captcha_solvers::ImageToText;

// From raw bytes (automatically base64-encoded)
let image_bytes = std::fs::read("captcha.png") ?;
let task = ImageToText::from_bytes(image_bytes);

// From pre-encoded base64 string
let task = ImageToText::from_base64("iVBORw0KGgoAAAANSUhEUgAA...");

// With options (for RuCaptcha)
let task = ImageToText::from_base64("base64data")
.case_sensitive()      // Answer is case-sensitive
.numbers_only()        // Answer contains only numbers
.with_min_length(4)    // Minimum 4 characters
.with_max_length(8)    // Maximum 8 characters
.with_comment("Enter red text only");  // Instruction for workers

// With module (for Capsolver)
let task = ImageToText::from_base64("base64data")
.with_module("common");  // "common" or "number"

let solution = service.solve_captcha(task).await?;
let text = solution.into_image_to_text().text();
println!("Recognized text: {}", text);
```

### Using Proxy

```rust
use captcha_solvers::ProxyConfig;

// HTTP
let proxy = ProxyConfig::http("host", 8080);

// SOCKS5 with authentication
let proxy = ProxyConfig::socks5("host", 1080).with_auth("user", "pass");

// Available types: http, https, socks4, socks5
```

### Retry Configuration

```rust
use captcha_solvers::{RetryConfig, CaptchaRetryableProvider, CaptchaSolverService};
use captcha_solvers::capsolver::CapsolverProvider;
use std::time::Duration;

let base_provider = CapsolverProvider::new("api_key")?;

let retry_config = RetryConfig::default()
    .with_max_retries(5)
    .with_min_delay(Duration::from_millis(500))
.with_max_delay(Duration::from_secs(30))
.with_factor(2.0);

// Wrap provider with retry logic and add a callback for retry notifications
let provider = CaptchaRetryableProvider::with_config(base_provider, retry_config)
.with_on_retry( | error, duration| {
eprintln ! (
"Retry triggered: will retry after {:?} due to error: {}",
duration, error
);
});

let service = CaptchaSolverService::new(provider);
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
cargo run --example image_to_text
cargo run --example with_proxy
cargo run --example with_retry
cargo run --example with_config
cargo run --example with_cancellation
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

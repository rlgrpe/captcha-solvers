# AGENTS.md

This document provides essential information for AI agents and developers working on this codebase.

## Quick Start

```bash
# Clone and setup
git clone https://github.com/rlgrpe/captcha-solvers.git
cd captcha-solvers
cp .env.example .env
# Edit .env with your API keys

# Build
cargo build --all-features

# Run tests
cargo test --all-features

# Run a specific example
cargo run --example basic_recaptcha_v2
```

## Build Commands

| Command | Description |
|---------|-------------|
| `cargo build` | Build with default features |
| `cargo build --all-features` | Build with all features enabled |
| `cargo build --release` | Build optimized release binary |

## Test Commands

| Command | Description |
|---------|-------------|
| `cargo test --all-features` | Run all tests |
| `cargo test --all-features -- --ignored` | Run integration tests (requires API keys) |
| `cargo test <name>` | Run tests matching name |

## Lint & Format

| Command | Description |
|---------|-------------|
| `cargo fmt --check` | Check formatting |
| `cargo fmt` | Apply formatting |
| `cargo clippy --all-features -- -D warnings` | Run linter with strict warnings |

## Project Structure

```
captcha-solvers/
├── src/
│   ├── lib.rs              # Main library entry point
│   ├── providers/          # Provider implementations (Capsolver, RuCaptcha)
│   ├── tasks/              # Captcha task types (ReCaptcha, Turnstile, etc.)
│   ├── service/            # Service layer and configuration
│   ├── solutions.rs        # Solution types
│   ├── errors.rs           # Error types
│   └── utils/              # Shared utilities (proxy, retry, serde)
├── tests/                  # Integration tests
├── examples/               # Usage examples
└── Cargo.toml              # Package manifest
```

## Features

| Feature | Description | Default |
|---------|-------------|---------|
| `capsolver` | Enable Capsolver provider | Yes |
| `rucaptcha` | Enable RuCaptcha provider | Yes |
| `tracing` | OpenTelemetry tracing instrumentation | Yes |
| `metrics` | OpenTelemetry metrics | No |
| `native-tls` | Native TLS backend | Yes |
| `rustls-tls` | Pure Rust TLS backend | No |

## Environment Variables

Required for integration tests:
- `CAPSOLVER_API_KEY` - Capsolver API key
- `RUCAPTCHA_API_KEY` - RuCaptcha API key

Optional proxy configuration:
- `PROXY_HOST`, `PROXY_PORT`, `PROXY_TYPE`, `PROXY_USER`, `PROXY_PASSWORD`

## Security Guidelines

### API Key Handling
- **Never log API keys** - Use the `secrecy` crate's `Secret<String>` type
- API keys are automatically redacted when using `Debug` formatting
- Store keys in environment variables, never in code

### Log Scrubbing
When logging request/response data:
- Redact `clientKey` and `apikey` fields
- Use structured logging with `tracing` for automatic field filtering
- Example: `tracing::info!(api_key = "[REDACTED]", "Creating task")`

## Troubleshooting

### Common Issues

1. **Tests fail with "API key not found"**
   - Ensure `.env` file exists with valid API keys
   - Run `source .env` or use `dotenvy` in tests

2. **Clippy warnings**
   - Run `cargo clippy --fix --all-features` to auto-fix
   - Check for `#[allow(dead_code)]` on intentionally unused items

3. **Integration tests timeout**
   - Use `CaptchaSolverServiceConfig::patient()` for slow providers
   - Check provider status pages for outages

## Contributing

1. Fork the repository
2. Create a feature branch
3. Run `cargo fmt` and `cargo clippy` before committing
4. Submit a pull request with clear description

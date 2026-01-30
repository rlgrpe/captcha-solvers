---
name: test
description: Run the test suite for captcha-solvers. Use when verifying changes, debugging test failures, or checking coverage.
---

# Test Skill

Run and manage tests for the captcha-solvers library.

## Quick Commands

| Command | Description |
|---------|-------------|
| `cargo test --all-features` | Run all unit tests |
| `cargo test --all-features -- --ignored` | Run integration tests (requires API keys) |
| `cargo test <name>` | Run tests matching name |
| `cargo test -- --nocapture` | Show println! output |

## Instructions

### 1. Run Unit Tests

```bash
cargo test --all-features
```

Unit tests use `wiremock` for mocking HTTP responses and don't require API keys.

### 2. Run Integration Tests (Optional)

Integration tests require real API keys:

```bash
# Set up environment
cp .env.example .env
# Edit .env with your API keys

# Run integration tests
cargo test --all-features -- --ignored
```

### 3. Run Specific Test

```bash
# By name
cargo test test_create_task

# By module
cargo test capsolver::

# With output
cargo test test_name -- --nocapture
```

### 4. Check Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --all-features --out html

# View report
open tarpaulin-report.html
```

### 5. Debug Failing Tests

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test failing_test

# Run single-threaded for clearer output
cargo test -- --test-threads=1
```

## Test Structure

```
tests/
├── common/mod.rs           # Shared test utilities
├── capsolver_integration.rs # Capsolver provider tests
└── rucaptcha_integration.rs # RuCaptcha provider tests

src/
├── providers/capsolver/tests.rs  # Unit tests for Capsolver
├── providers/rucaptcha/tests.rs  # Unit tests for RuCaptcha
└── */mod.rs                      # Module-level tests
```

## Verification

All tests should pass before committing:

```bash
cargo test --all-features && echo "All tests passed!"
```

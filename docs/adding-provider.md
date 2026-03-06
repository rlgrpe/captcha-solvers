# Adding a New Provider

Step-by-step guide for adding a new captcha solving provider to the library.

## Prerequisites

Read [architecture.md](architecture.md) first to understand the task conversion pipeline and core traits.

## 1. Add the Feature Flag

In `Cargo.toml`:

```toml
[features]
default = ["capsolver", "capmonster", "rucaptcha", "your_provider", "tracing", "native-tls"]
your_provider = []
```

## 2. Create the Module Structure

```
src/providers/your_provider/
├── mod.rs          # Module docs, re-exports
├── provider.rs     # Provider struct + Provider trait impl
├── types.rs        # ProviderTask enum + From/TryFrom + ProviderSolution enum
├── errors.rs       # Error type + RetryableError impl
├── response.rs     # API response parsing helpers
└── tests.rs        # wiremock-based integration tests
```

## 3. Define Error Types (`errors.rs`)

Your error type must implement `std::error::Error`, `Send`, `Sync`, and `RetryableError`:

```rust
use crate::errors::{RetryableError, UnsupportedTaskError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum YourProviderError {
    #[error("Failed to build HTTP client: {0}")]
    BuildHttpClient(#[source] reqwest::Error),

    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest_middleware::Error),

    #[error("Failed to parse response: {0}")]
    ParseResponse(#[source] reqwest::Error),

    #[error("API error: {0}")]
    Api(#[source] YourApiError),

    #[error("{0}")]
    UnsupportedTask(#[source] UnsupportedTaskError),

    #[error("Invalid task data: {0}")]
    InvalidTaskData(String),
}

pub type Result<T> = std::result::Result<T, YourProviderError>;

impl RetryableError for YourProviderError {
    fn is_retryable(&self) -> bool {
        match self {
            // Network errors are transient
            Self::HttpRequest(_) => true,
            // Delegate to API error code classification
            Self::Api(e) => e.error_code.is_retryable(),
            // Everything else is permanent
            _ => false,
        }
    }

    fn should_retry_operation(&self) -> bool {
        match self {
            Self::HttpRequest(_) => true,
            Self::Api(e) => e.error_code.should_retry_operation(),
            _ => false,
        }
    }
}
```

Key points:
- **`is_retryable()`** — `true` for transient errors (network timeouts, rate limits, temporary bans). The service will keep polling the same `task_id`.
- **`should_retry_operation()`** — `true` if a fresh `solve_captcha()` call might succeed (captcha unsolvable, task expired). `false` for permanent errors (invalid API key, zero balance).
- Always include `UnsupportedTask(UnsupportedTaskError)` — needed for the `TryFrom` conversion in `types.rs`.

## 4. Define Provider Task Types (`types.rs`)

### 4a. Provider-Specific Task Enum

Map each supported captcha to a serde-serializable variant matching the provider's API:

```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum YourProviderTask {
    NoCaptchaTaskProxyless {
        #[serde(rename = "websiteURL")]
        website_url: String,
        #[serde(rename = "websiteKey")]
        website_key: String,
        #[serde(rename = "isInvisible", skip_serializing_if = "Option::is_none")]
        is_invisible: Option<bool>,
    },
    // ... more variants matching the provider's API
}
```

### 4b. Implement `TryFrom<CaptchaTask>`

This is the critical conversion step. Use `From` when conversion is infallible, `TryFrom` when the provider doesn't support certain tasks or fields:

```rust
use crate::errors::UnsupportedTaskError;
use crate::tasks::CaptchaTask;

impl TryFrom<CaptchaTask> for YourProviderTask {
    type Error = UnsupportedTaskError;

    fn try_from(task: CaptchaTask) -> Result<Self, Self::Error> {
        match task {
            CaptchaTask::ReCaptchaV2(t) => Ok(t.into()),
            CaptchaTask::ReCaptchaV3(t) => t.try_into(),
            CaptchaTask::Turnstile(t) => Ok(t.into()),
            // Return UnsupportedTaskError for unsupported task types
            CaptchaTask::CloudflareChallenge(_) => {
                Err(UnsupportedTaskError::new("CloudflareChallenge", "YourProvider"))
            }
            CaptchaTask::ImageToText(t) => t.try_into(),
            // ... handle all variants
        }
    }
}
```

For individual task conversions, validate field support:

```rust
impl TryFrom<crate::tasks::ReCaptchaV3> for YourProviderTask {
    type Error = UnsupportedTaskError;

    fn try_from(task: crate::tasks::ReCaptchaV3) -> Result<Self, Self::Error> {
        // Reject unsupported field combinations
        let mut unsupported = Vec::new();
        if task.proxy.is_some() {
            unsupported.push("proxy");
        }
        if task.enterprise_payload.is_some() {
            unsupported.push("enterprise_payload");
        }
        if !unsupported.is_empty() {
            return Err(UnsupportedTaskError::unsupported_fields(
                "ReCaptchaV3", "YourProvider", unsupported,
            ));
        }

        Ok(Self::RecaptchaV3 {
            website_url: task.website_url,
            website_key: task.website_key,
            min_score: task.min_score,
            page_action: task.page_action,
        })
    }
}
```

### 4c. Define Solution Types

Re-use shared solution types and wrap them in a provider-specific enum:

```rust
use serde::Deserialize;

// Re-export shared types
pub use crate::solutions::{ImageToTextSolution, ReCaptchaSolution, TurnstileSolution};

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum YourProviderSolution {
    // Order matters for untagged deserialization — put the most
    // distinctive variant first (unique field names)
    ImageToText(ImageToTextSolution),
    ReCaptcha(ReCaptchaSolution),
    Turnstile(TurnstileSolution),
}

impl crate::solutions::ProviderSolution for YourProviderSolution {}

// Add accessor methods: as_recaptcha(), into_recaptcha(), etc.
impl YourProviderSolution {
    pub fn into_recaptcha(self) -> ReCaptchaSolution {
        match self {
            Self::ReCaptcha(s) => s,
            _ => panic!("Expected ReCaptcha solution"),
        }
    }
    // ... similar for turnstile, image_to_text
}
```

## 5. Implement the Provider (`provider.rs`)

```rust
use crate::providers::traits::{Provider, TaskCreationOutcome};
use crate::tasks::CaptchaTask;
use crate::utils::types::TaskId;
use reqwest::Url;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use secrecy::{ExposeSecret, SecretString};

pub const DEFAULT_API_URL: &str = "https://api.yourprovider.com";

#[derive(Clone)]
pub struct YourProvider {
    http_client: ClientWithMiddleware,
    api_key: SecretString,
    url: Url,
}

impl Provider for YourProvider {
    type Solution = YourProviderSolution;
    type Error = YourProviderError;

    async fn create_task(
        &self,
        task: CaptchaTask,
    ) -> Result<TaskCreationOutcome<Self::Solution>, Self::Error> {
        // 1. Convert shared task to provider-specific format
        let internal_task: YourProviderTask = task
            .try_into()
            .map_err(YourProviderError::UnsupportedTask)?;

        // 2. Optional: validate provider-specific constraints
        Self::validate_task(&internal_task)?;

        // 3. Send to API
        let task_id = self.create_task_internal(internal_task).await?;

        // 4. Return outcome
        // Most providers return Pending — use Ready for sync tasks (e.g. ImageToText)
        Ok(TaskCreationOutcome::Pending(task_id))
    }

    async fn get_task_result(
        &self,
        task_id: &TaskId,
    ) -> Result<Option<Self::Solution>, Self::Error> {
        self.get_task_result_internal(task_id).await
    }
}
```

Key requirements:
- The struct must be `Clone + Send + Sync`.
- Store API key as `SecretString` (from `secrecy` crate).
- Use `reqwest_middleware::ClientWithMiddleware` for the HTTP client.
- Use `TaskCreationOutcome::Ready` for task types that return solutions immediately (like Capsolver's ImageToText).

## 6. Wire Up the Module

### `src/providers/your_provider/mod.rs`

```rust
mod errors;
mod provider;
mod response;
mod types;

#[cfg(test)]
mod tests;

pub use errors::{YourProviderError, YourApiError, YourErrorCode};
pub use provider::{YourProvider, YourProviderBuilder, DEFAULT_API_URL};
pub use types::{YourProviderSolution, ReCaptchaSolution, TurnstileSolution, ImageToTextSolution};
pub use crate::utils::proxy::{ProxyConfig, ProxyType};
```

### `src/providers/mod.rs`

```rust
#[cfg(feature = "your_provider")]
pub mod your_provider;
```

### `src/lib.rs`

Add the feature-gated public module:

```rust
#[cfg(feature = "your_provider")]
pub mod your_provider {
    //! YourProvider implementation.
    pub use crate::providers::your_provider::*;
}
```

## 7. Proxy Serialization

If the provider uses the same proxy format as Capsolver/CapMonster (separate fields), reuse `ApiProxyFields`:

```rust
use crate::utils::proxy::ApiProxyFields;

// In your task variant:
#[serde(flatten, skip_serializing_if = "Option::is_none")]
proxy: Option<ApiProxyFields>,

// In the From impl:
let proxy = task.proxy.map(|p| p.into_api_proxy_fields());
```

If the provider has a different format (like RuCaptcha mapping https→http), create a new fields type in `src/utils/proxy.rs` — see `RucaptchaProxyFields` for reference.

## 8. Write Tests (`tests.rs`)

Use `wiremock` to mock the provider API:

```rust
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};
use serde_json::json;

fn mock_provider(server: &MockServer) -> YourProvider {
    YourProvider::builder("test_api_key")
        .url(Url::parse(&server.uri()).unwrap())
        .build()
        .unwrap()
}

#[tokio::test]
async fn test_create_task_success() {
    let server = MockServer::start().await;
    let provider = mock_provider(&server);

    Mock::given(method("POST"))
        .and(path("/createTask"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "errorId": 0,
            "taskId": "12345"
        })))
        .mount(&server)
        .await;

    let task = ReCaptchaV2::new("https://example.com", "site-key");
    let outcome = provider.create_task(task.into()).await.unwrap();
    assert!(outcome.is_pending());
}
```

Also add serialization tests for `TryFrom` conversions (see `src/providers/capmonster/types.rs` tests for patterns).

## 9. Add an Example

Create `examples/your_provider.rs`:

```rust
use captcha_solvers::{
    CaptchaSolverService, CaptchaSolverServiceTrait,
    ReCaptchaV2,
    your_provider::YourProvider,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = YourProvider::new("your_api_key")?;
    let service = CaptchaSolverService::new(provider);

    let task = ReCaptchaV2::new("https://example.com", "site_key");
    let solution = service.solve_captcha(task).await?;
    println!("Token: {}", solution.into_recaptcha().token());

    Ok(())
}
```

## 10. Update Documentation

1. Update the support matrix in `docs/architecture.md`
2. Update `src/lib.rs` top-level doc comment (providers table, captcha types table)
3. Update `README.md` if it lists providers

## Checklist

- [ ] Feature flag in `Cargo.toml`
- [ ] Error type with `RetryableError` impl (two-level retryability)
- [ ] Provider task enum with `TryFrom<CaptchaTask>` (return `UnsupportedTaskError` for unsupported tasks/fields)
- [ ] Solution enum with `ProviderSolution` marker trait
- [ ] Provider struct implementing `Provider` trait (`Clone + Send + Sync`)
- [ ] API key stored as `SecretString`
- [ ] Builder pattern for provider construction
- [ ] Feature-gated in `providers/mod.rs` and `lib.rs`
- [ ] Proxy serialization (reuse `ApiProxyFields` or add new format)
- [ ] `#[cfg(feature = "tracing")]` instrumentation on `create_task` / `get_task_result`
- [ ] wiremock tests for create_task, get_task_result, error handling
- [ ] Serialization tests for all `TryFrom` conversions
- [ ] Example in `examples/`
- [ ] `cargo clippy --all-features -- -D warnings` passes
- [ ] `cargo test --all-features` passes
- [ ] Documentation updated

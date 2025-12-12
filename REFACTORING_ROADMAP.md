# Refactoring Roadmap: captcha-solvers

## Executive Summary

This roadmap outlines a systematic refactoring of the `captcha-solvers` crate to eliminate code duplication, improve maintainability, and reduce the cost of adding new providers.

**Current State:**
- ~6,500 lines of Rust code
- 2 providers (Capsolver, RuCaptcha) with ~97% structural duplication
- ~2,830 lines of duplicated code (43% of provider code)

**Target State:**
- ~4,000 lines of Rust code (38% reduction)
- Shared abstractions for common patterns
- Adding a new provider requires ~400 lines (down from ~1,860)

---

## Phase 1: Extract Shared Proxy Configuration

**Goal:** Eliminate proxy field duplication across task variants.

### Problem

Every task variant with proxy support duplicates 5 fields:

```rust
// Repeated in 10+ variants per provider
#[serde(rename = "proxyType", serialize_with = "...")]
proxy_type: ProxyType,
#[serde(rename = "proxyAddress")]
proxy_address: String,
#[serde(rename = "proxyPort")]
proxy_port: u16,
#[serde(rename = "proxyLogin", skip_serializing_if = "...")]
proxy_login: Option<String>,
#[serde(rename = "proxyPassword", skip_serializing_if = "...")]
proxy_password: Option<String>,
```

### Solution

Create a flattened proxy struct in `src/proxy.rs`:

```rust
/// Proxy fields for serialization into task payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyFields {
    #[serde(serialize_with = "serialize_proxy_type")]
    pub proxy_type: ProxyType,
    pub proxy_address: String,
    pub proxy_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_login: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_password: Option<String>,
}

impl From<ProxyConfig> for ProxyFields {
    fn from(config: ProxyConfig) -> Self { /* ... */ }
}
```

### Tasks

- [ ] Add `ProxyFields` struct to `src/proxy.rs`
- [ ] Add `From<ProxyConfig>` implementation
- [ ] Update Capsolver task variants to use `#[serde(flatten)] proxy: ProxyFields`
- [ ] Update RuCaptcha task variants to use `#[serde(flatten)] proxy: ProxyFields`
- [ ] Update constructors to accept `ProxyConfig` and convert internally
- [ ] Run tests to verify serialization unchanged

### Impact

- **Lines removed:** ~150
- **Risk:** Low (serialization output unchanged)

---

## Phase 2: Unify Response Handling

**Goal:** Create a single generic response parser for all providers.

### Problem

Both providers have nearly identical response parsing:

```rust
// capsolver/response.rs (~46 lines)
pub enum CapsolverResponse<T> {
    Success(T),
    Error(CapsolverError),
}

// rucaptcha/response.rs (~49 lines)
pub enum RucaptchaResponse<T> {
    Success(T),
    Error(RucaptchaError),
}
```

### Solution

Create `src/response.rs` with generic response handling:

```rust
/// Generic API response wrapper
#[derive(Debug)]
pub enum ApiResponse<T, E> {
    Success(T),
    Error(E),
}

impl<T, E> ApiResponse<T, E> {
    pub fn into_result(self) -> Result<T, E> {
        match self {
            Self::Success(data) => Ok(data),
            Self::Error(err) => Err(err),
        }
    }
}

/// Trait for provider-specific response deserialization
pub trait ResponseFormat: Sized {
    type Error: std::error::Error;

    fn deserialize_response<'de, D, T>(deserializer: D) -> Result<ApiResponse<T, Self::Error>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: serde::Deserialize<'de>;
}
```

### Tasks

- [ ] Create `src/response.rs` with `ApiResponse<T, E>` enum
- [ ] Add `ResponseFormat` trait for provider-specific deserialization
- [ ] Implement `ResponseFormat` for Capsolver
- [ ] Implement `ResponseFormat` for RuCaptcha
- [ ] Update provider clients to use shared response types
- [ ] Remove `capsolver/response.rs` and `rucaptcha/response.rs`
- [ ] Run tests

### Impact

- **Lines removed:** ~60
- **Risk:** Low

---

## Phase 3: Create Generic API Client

**Goal:** Extract common HTTP client logic into a reusable abstraction.

### Problem

Both providers have nearly identical client implementations:

```rust
// capsolver/client.rs (~270 lines)
impl CapsolverClient {
    pub async fn create_task(&self, task: CapsolverTask) -> Result<...> {
        let response = self.client.post(url).json(&request).send().await?;
        // parse response...
    }

    pub async fn get_task_result(&self, task_id: &TaskId) -> Result<...> {
        let response = self.client.post(url).json(&request).send().await?;
        // parse response...
    }
}

// rucaptcha/client.rs (~270 lines) - IDENTICAL STRUCTURE
```

### Solution

Create `src/client.rs` with a generic client:

```rust
/// Configuration for provider-specific API details
pub trait ApiConfig: Send + Sync + Clone {
    type Task: Serialize + Send + Sync;
    type Solution: DeserializeOwned + Send + Sync;
    type Error: std::error::Error + RetryableError + DeserializeOwned + Send + Sync + 'static;

    /// Base URL for the API
    fn base_url(&self) -> &Url;

    /// API key for authentication
    fn api_key(&self) -> &str;

    /// Path for creating tasks (e.g., "/createTask")
    fn create_task_path() -> &'static str;

    /// Path for getting results (e.g., "/getTaskResult")
    fn get_result_path() -> &'static str;

    /// Build the create task request body
    fn build_create_request(&self, task: &Self::Task) -> serde_json::Value;

    /// Build the get result request body
    fn build_get_result_request(&self, task_id: &TaskId) -> serde_json::Value;

    /// Parse task ID from create response
    fn parse_task_id(response: &serde_json::Value) -> Result<TaskId, Self::Error>;

    /// Parse solution from get result response (None if not ready)
    fn parse_solution(response: &serde_json::Value) -> Result<Option<Self::Solution>, Self::Error>;
}

/// Generic API client that works with any provider
#[derive(Clone)]
pub struct GenericClient<C: ApiConfig> {
    config: C,
    http_client: reqwest::Client,
}

impl<C: ApiConfig> GenericClient<C> {
    pub async fn create_task(&self, task: C::Task) -> Result<TaskId, C::Error> { /* ... */ }
    pub async fn get_task_result(&self, task_id: &TaskId) -> Result<Option<C::Solution>, C::Error> { /* ... */ }
}
```

### Tasks

- [ ] Create `src/client.rs` with `ApiConfig` trait
- [ ] Implement `GenericClient<C>` struct
- [ ] Create `CapsolverConfig` implementing `ApiConfig`
- [ ] Create `RucaptchaConfig` implementing `ApiConfig`
- [ ] Update `CapsolverClient` to wrap `GenericClient<CapsolverConfig>`
- [ ] Update `RucaptchaClient` to wrap `GenericClient<RucaptchaConfig>`
- [ ] Preserve existing public API (builder pattern, etc.)
- [ ] Run tests

### Impact

- **Lines removed:** ~400
- **Risk:** Medium (HTTP layer changes)

---

## Phase 4: Consolidate Task Types

**Goal:** Extract common captcha task definitions with provider-specific serialization.

### Problem

Task enums are massive and nearly identical:

```rust
// capsolver/types.rs (~797 lines)
pub enum CapsolverTask {
    ReCaptchaV2Task { website_url, website_key, ... },
    ReCaptchaV2TaskProxyLess { website_url, website_key, ... },
    ReCaptchaV2EnterpriseTask { ... },
    // 20+ more variants...
}

// rucaptcha/types.rs (~780 lines) - SAME VARIANTS, DIFFERENT NAMING
```

### Solution

Create shared task definitions in `src/tasks/`:

```rust
// src/tasks/mod.rs
pub mod recaptcha;
pub mod hcaptcha;
pub mod turnstile;
pub mod funcaptcha;
pub mod geetest;
pub mod image;

// src/tasks/recaptcha.rs
/// ReCaptcha V2 task parameters
#[derive(Debug, Clone)]
pub struct ReCaptchaV2 {
    pub website_url: String,
    pub website_key: String,
    pub page_action: Option<String>,
    pub is_invisible: bool,
    pub is_enterprise: bool,
    pub enterprise_payload: Option<serde_json::Value>,
    pub api_domain: Option<String>,
    pub proxy: Option<ProxyConfig>,
}

impl ReCaptchaV2 {
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self { /* ... */ }
    pub fn invisible(mut self) -> Self { /* ... */ }
    pub fn enterprise(mut self) -> Self { /* ... */ }
    pub fn with_proxy(mut self, proxy: ProxyConfig) -> Self { /* ... */ }
}
```

Provider-specific adapters handle serialization:

```rust
// capsolver/types.rs (reduced to ~200 lines)
impl From<ReCaptchaV2> for CapsolverTask {
    fn from(task: ReCaptchaV2) -> Self {
        // Convert to provider-specific format
    }
}

impl Serialize for CapsolverTask {
    // Capsolver-specific serialization (camelCase, specific field names)
}
```

### Tasks

- [ ] Create `src/tasks/` module structure
- [ ] Define `ReCaptchaV2`, `ReCaptchaV3` in `src/tasks/recaptcha.rs`
- [ ] Define `HCaptcha` in `src/tasks/hcaptcha.rs`
- [ ] Define `Turnstile` in `src/tasks/turnstile.rs`
- [ ] Define `FunCaptcha` in `src/tasks/funcaptcha.rs`
- [ ] Define `GeeTest` variants in `src/tasks/geetest.rs`
- [ ] Define `ImageToText` in `src/tasks/image.rs`
- [ ] Create `CapsolverTask` as thin wrapper with `From` impls
- [ ] Create `RucaptchaTask` as thin wrapper with `From` impls
- [ ] Update constructors to use shared types
- [ ] Maintain backward compatibility via re-exports
- [ ] Run tests

### Impact

- **Lines removed:** ~1,200
- **Risk:** Medium-High (API surface changes)

---

## Phase 5: Implement Task Builder Pattern

**Goal:** Replace 13+ constructors per provider with fluent builders.

### Problem

Each provider has repetitive constructors:

```rust
impl CapsolverTask {
    pub fn recaptcha_v2(url: &str, key: &str) -> Self { /* ... */ }
    pub fn recaptcha_v2_invisible(url: &str, key: &str) -> Self { /* ... */ }
    pub fn recaptcha_v2_enterprise(url: &str, key: &str) -> Self { /* ... */ }
    pub fn recaptcha_v2_with_proxy(url: &str, key: &str, proxy: ProxyConfig) -> Self { /* ... */ }
    // 10+ more...
}
```

### Solution

Use builder pattern from Phase 4 shared types:

```rust
// Usage becomes:
let task = ReCaptchaV2::new("https://example.com", "site-key")
    .invisible()
    .with_proxy(proxy)
    .into(); // Into<CapsolverTask>

// Or using provider-specific builder
let task = CapsolverTask::recaptcha_v2("https://example.com", "site-key")
    .invisible()
    .with_proxy(proxy);
```

### Tasks

- [ ] Add builder methods to shared task types
- [ ] Create `TaskBuilder` trait for common builder interface
- [ ] Update provider task enums to accept shared types via `From`
- [ ] Deprecate old constructors (keep for backward compat initially)
- [ ] Update documentation with new patterns
- [ ] Run tests

### Impact

- **Lines removed:** ~200
- **Risk:** Low (additive change)

---

## Phase 6: Standardize Error Handling

**Goal:** Unify error code handling across providers.

### Problem

Capsolver uses manual `FromStr`, RuCaptcha uses serde attributes:

```rust
// Capsolver - manual parsing (~60 lines)
impl FromStr for CapsolverErrorCode {
    fn from_str(s: &str) -> Self {
        match s {
            "ERROR_SERVICE_UNAVALIABLE" => Self::ServiceUnavailable,
            // ...
        }
    }
}

// RuCaptcha - serde attributes (cleaner)
#[derive(Deserialize)]
pub enum RucaptchaErrorCode {
    #[serde(rename = "ERROR_SERVICE_UNAVAILABLE")]
    ServiceUnavailable,
    // ...
}
```

### Solution

Standardize on serde attributes for both providers:

```rust
// Common error trait
pub trait ProviderError: std::error::Error + RetryableError {
    fn error_code(&self) -> &str;
    fn error_description(&self) -> Option<&str>;
}

// Provider errors use serde for parsing
#[derive(Debug, Deserialize, thiserror::Error)]
pub enum CapsolverErrorCode {
    #[serde(rename = "ERROR_SERVICE_UNAVALIABLE")]
    #[error("Service unavailable")]
    ServiceUnavailable,
    // ...
}
```

### Tasks

- [ ] Create `ProviderError` trait in `src/errors.rs`
- [ ] Refactor `CapsolverErrorCode` to use serde attributes
- [ ] Ensure `RucaptchaErrorCode` follows same pattern
- [ ] Remove manual `FromStr` implementations
- [ ] Run tests

### Impact

- **Lines removed:** ~60
- **Risk:** Low

---

## Phase 7: Consolidate Serialization Helpers

**Goal:** Centralize custom serde functions.

### Problem

Duplicate serializers across providers:

```rust
// Both providers have similar functions:
fn serialize_proxy_type<S>(...) -> Result<S::Ok, S::Error> { /* ... */ }
fn skip_if_false<T>(...) -> bool { /* ... */ }
```

### Solution

Create `src/serde_helpers.rs`:

```rust
pub mod proxy {
    pub fn serialize_proxy_type<S>(proxy_type: &ProxyType, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
    {
        serializer.serialize_str(proxy_type.as_str())
    }
}

pub mod skip {
    pub fn if_false(value: &bool) -> bool { !*value }
    pub fn if_none<T>(value: &Option<T>) -> bool { value.is_none() }
}

pub mod deserialize {
    pub fn string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
    where D: Deserializer<'de>
    { /* ... */ }
}
```

### Tasks

- [ ] Create `src/serde_helpers.rs`
- [ ] Move shared serializers from both providers
- [ ] Update imports in provider types
- [ ] Remove duplicate functions from provider modules
- [ ] Run tests

### Impact

- **Lines removed:** ~80
- **Risk:** Low

---

## Phase 8: Create Shared Test Infrastructure

**Goal:** Reduce test code duplication and improve test maintainability.

### Problem

Each provider has ~290 lines of similar tests:

```rust
// capsolver/tests.rs
#[test]
fn test_recaptcha_v2_serialization() { /* ... */ }

// rucaptcha/tests.rs
#[test]
fn test_recaptcha_v2_serialization() { /* ... */ }
```

### Solution

Create shared test utilities:

```rust
// tests/common/mod.rs
pub mod assertions {
    pub fn assert_json_field<T: Serialize>(value: &T, field: &str, expected: &str) {
        let json = serde_json::to_value(value).unwrap();
        assert_eq!(json[field].as_str(), Some(expected));
    }

    pub fn assert_task_type<T: Serialize>(task: &T, expected_type: &str) {
        assert_json_field(task, "type", expected_type);
    }
}

pub mod fixtures {
    pub fn sample_proxy() -> ProxyConfig { /* ... */ }
    pub fn sample_recaptcha_params() -> (String, String) { /* ... */ }
}
```

### Tasks

- [ ] Expand `tests/common/mod.rs` with shared assertions
- [ ] Add fixture generators for common test data
- [ ] Refactor Capsolver tests to use shared utilities
- [ ] Refactor RuCaptcha tests to use shared utilities
- [ ] Add property-based tests for serialization roundtrips
- [ ] Run full test suite

### Impact

- **Lines removed:** ~100
- **Risk:** Low (test-only changes)

---

## Phase 9: Documentation and API Polish

**Goal:** Update documentation to reflect new architecture.

### Tasks

- [ ] Update `README.md` with new usage patterns
- [ ] Add module-level documentation for `src/tasks/`
- [ ] Add module-level documentation for `src/client.rs`
- [ ] Create migration guide for existing users
- [ ] Add examples for common use cases
- [ ] Generate and review rustdoc output

### Impact

- **Risk:** None (documentation only)

---

## Implementation Order

```
Phase 1 ──► Phase 2 ──► Phase 3 ──────────────────────────────────►
   │           │           │                                        │
   │           │           └─► Phase 4 ──► Phase 5 ──────────────► │
   │           │                  │                                 │
   │           └──────────────────┼─► Phase 6 ──► Phase 7 ────────►├──► Phase 9
   │                              │                                 │
   └──────────────────────────────┴─► Phase 8 ─────────────────────►
```

**Recommended order:**
1. **Phase 1** (Proxy) - Low risk, immediate wins
2. **Phase 2** (Response) - Low risk, sets up Phase 3
3. **Phase 6** (Errors) - Low risk, independent
4. **Phase 7** (Serializers) - Low risk, independent
5. **Phase 3** (Client) - Medium risk, enables Phase 4
6. **Phase 4** (Tasks) - High impact, core refactoring
7. **Phase 5** (Builders) - Builds on Phase 4
8. **Phase 8** (Tests) - Can run in parallel with others
9. **Phase 9** (Docs) - Final polish

---

## Success Metrics

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Total lines of code | ~6,500 | ~4,000 | -38% |
| Lines per new provider | ~1,860 | ~400 | -78% |
| Duplicated code | 43% | <10% | -33pp |
| Task type variants | 2 × 25 | 25 shared | -50% |
| Test utilities | None | Shared | ✓ |

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking API changes | High | Deprecate old APIs, maintain for 1-2 versions |
| Serialization regressions | High | Comprehensive JSON snapshot tests |
| Performance regression | Medium | Benchmark critical paths before/after |
| Feature flag complexity | Low | Clear documentation, integration tests |

---

## Future Considerations

After completing this roadmap:

1. **Add more providers** (2Captcha, Anti-Captcha, etc.) using new abstractions
2. **Provider-agnostic task API** - Users define tasks once, serialize for any provider
3. **Async trait stabilization** - Simplify when `async fn` in traits stabilizes
4. **Macro-based task generation** - Derive macros for task variants
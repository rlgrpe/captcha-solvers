# Architecture

## Overview

`captcha-solvers` is a provider-agnostic captcha solving library. Users work with shared task types and a unified service layer; the library routes tasks to provider-specific APIs via the `Provider` trait.

## High-Level Flow

```mermaid
flowchart TD
    subgraph User Code
        UT[/"ReCaptchaV2::new(...)<br>.invisible()<br>.enterprise()"/]
    end

    subgraph Service Layer
        SVC["CaptchaSolverService&lt;P&gt;"]
        POLL["Polling Loop<br>timeout · poll_interval · cancellation"]
    end

    subgraph Provider Layer
        RETRY["CaptchaRetryableProvider&lt;P&gt;<br><i>(optional wrapper)</i>"]
        PROV["Provider trait<br>create_task() → TaskCreationOutcome<br>get_task_result() → Option&lt;Solution&gt;"]
    end

    subgraph Concrete Providers
        CAP["CapsolverProvider"]
        CM["CapmonsterProvider"]
        RC["RucaptchaProvider"]
    end

    UT -- "Into&lt;CaptchaTask&gt;" --> SVC
    SVC --> POLL
    POLL -- "create_task / get_task_result" --> RETRY
    RETRY -- "delegates" --> PROV
    PROV --> CAP & CM & RC
    CAP & CM & RC -- "Solution / ServiceError" --> SVC
    SVC -- "Solution" --> UT
```

## Task Conversion Pipeline

Each shared task type goes through a two-step conversion before reaching the provider API:

```mermaid
flowchart LR
    A["UserTask<br><i>(ReCaptchaV2, Turnstile, …)</i>"] -- "Into&lt;CaptchaTask&gt;" --> B["CaptchaTask<br><i>(enum wrapper)</i>"]
    B -- "TryFrom&lt;CaptchaTask&gt;<br>for ProviderTask" --> C["ProviderTask<br><i>(CapmonsterTask, CapsolverTask, …)</i>"]
    C -- "serde Serialize" --> D["JSON payload<br>→ HTTP POST"]
```

- `Into<CaptchaTask>` — infallible, wraps the user task in the unified enum.
- `TryFrom<CaptchaTask> for ProviderTask` — fallible. Returns `UnsupportedTaskError` if the provider doesn't support the task type or specific field combinations.

## Core Types

```mermaid
classDiagram
    class Provider {
        <<trait>>
        +type Solution: ProviderSolution
        +type Error: RetryableError
        +create_task(CaptchaTask) TaskCreationOutcome~Solution~
        +get_task_result(TaskId) Option~Solution~
    }

    class CaptchaSolverServiceTrait {
        <<trait>>
        +type Solution: ProviderSolution
        +solve_captcha(T: Into~CaptchaTask~) Solution
        +solve_captcha_cancellable(T, CancellationToken) Solution
    }

    class CaptchaSolverService~P~ {
        -provider: P
        -config: CaptchaSolverServiceConfig
        +new(P) Self
        +with_config(P, Config) Self
        +builder(P) Builder
    }

    class CaptchaRetryableProvider~P~ {
        -inner: Arc~P~
        -retry_config: RetryConfig
        -on_retry: Option~Callback~
        +new(P) Self
        +with_config(P, RetryConfig) Self
        +with_on_retry(Fn) Self
    }

    class TaskCreationOutcome~S~ {
        <<enum>>
        Pending(TaskId)
        Ready(TaskId, S)
    }

    class ServiceError {
        <<enum>>
        Provider
        SolutionTimeout
        Cancelled
    }

    class RetryableError {
        <<trait>>
        +is_retryable() bool
        +should_retry_operation() bool
    }

    CaptchaSolverService ..|> CaptchaSolverServiceTrait
    CaptchaSolverService --> Provider : uses
    CaptchaRetryableProvider ..|> Provider : implements
    CaptchaRetryableProvider --> Provider : wraps
    Provider --> TaskCreationOutcome : returns
    ServiceError ..|> RetryableError
```

## CaptchaTask Enum

```mermaid
classDiagram
    class CaptchaTask {
        <<enum>>
        ReCaptchaV2(ReCaptchaV2)
        ReCaptchaV3(ReCaptchaV3)
        Turnstile(Turnstile)
        TurnstileChallenge(TurnstileChallenge)
        TurnstileWaitRoom(TurnstileWaitRoom)
        CloudflareChallenge(CloudflareChallenge)
        ImageToText(ImageToText)
    }

    class ReCaptchaV2 {
        +website_url: String
        +website_key: String
        +is_invisible: bool
        +is_enterprise: bool
        +proxy: Option~ProxyConfig~
    }

    class ReCaptchaV3 {
        +website_url: String
        +website_key: String
        +min_score: Option~f32~
        +is_enterprise: bool
    }

    class Turnstile {
        +website_url: String
        +website_key: String
        +action: Option~String~
        +cdata: Option~String~
    }

    CaptchaTask --> ReCaptchaV2
    CaptchaTask --> ReCaptchaV3
    CaptchaTask --> Turnstile
```

## Solving Lifecycle (Sequence)

```mermaid
sequenceDiagram
    participant User
    participant Service as CaptchaSolverService
    participant Provider as Provider (impl)
    participant API as Provider API

    User->>Service: solve_captcha(task)
    Service->>Service: task.into() → CaptchaTask
    Service->>Provider: create_task(CaptchaTask)
    Provider->>Provider: TryFrom → ProviderTask
    Provider->>API: POST /createTask
    API-->>Provider: task_id

    alt TaskCreationOutcome::Ready
        Provider-->>Service: Ready { task_id, solution }
        Service-->>User: Ok(solution)
    else TaskCreationOutcome::Pending
        Provider-->>Service: Pending(task_id)
        loop poll_interval until timeout
            Service->>Provider: get_task_result(task_id)
            Provider->>API: POST /getTaskResult
            alt solution ready
                API-->>Provider: solution
                Provider-->>Service: Some(solution)
                Service-->>User: Ok(solution)
            else not ready
                API-->>Provider: processing
                Provider-->>Service: None
                Service->>Service: sleep(poll_interval)
            else transient error
                API-->>Provider: error (retryable)
                Provider-->>Service: Err (logged, continue)
            else permanent error
                API-->>Provider: error (permanent)
                Provider-->>Service: Err
                Service-->>User: Err(ServiceError::Provider)
            end
        end
        Service-->>User: Err(ServiceError::SolutionTimeout)
    end
```

## Error Classification

```mermaid
flowchart TD
    ERR["Error occurs"]
    ERR --> RETRY{"is_retryable()?"}
    RETRY -- "true" --> SAME["Retry same task_id<br><i>(network timeout, rate limit)</i>"]
    RETRY -- "false" --> FRESH{"should_retry_operation()?"}
    FRESH -- "true" --> NEW["Create fresh task<br><i>(captcha unsolvable, task timeout)</i>"]
    FRESH -- "false" --> FAIL["Permanent failure<br><i>(invalid API key, zero balance)</i>"]
```

Two-level retryability:
- **`is_retryable()`** — same task_id can be polled again (transient network errors).
- **`should_retry_operation()`** — a brand new `solve_captcha()` call might succeed (captcha was unsolvable, but next one might work).

## Provider Support Matrix

| Task Type | Capsolver | CapMonster | RuCaptcha |
|-----------|-----------|------------|-----------|
| ReCaptchaV2 | Yes | Yes | Yes |
| ReCaptchaV3 | Yes | Yes | Yes |
| Turnstile | Yes | Yes | Yes |
| TurnstileChallenge | — | Yes | — |
| TurnstileWaitRoom | — | Yes | — |
| CloudflareChallenge | Yes | — | — |
| ImageToText | Yes | Yes | Yes |

## Module Layout

```
src/
├── lib.rs                      # Public API re-exports
├── errors.rs                   # UnsupportedTaskError, RetryableError trait
├── solutions.rs                # Shared solution types (ReCaptchaSolution, TurnstileSolution, …)
├── tasks/                      # Shared task types with builder pattern
│   ├── mod.rs                  # CaptchaTask enum + From impls
│   ├── recaptcha.rs            # ReCaptchaV2, ReCaptchaV3
│   ├── cloudflare.rs           # Turnstile, CloudflareChallenge
│   ├── turnstile_challenge.rs  # TurnstileChallenge
│   ├── turnstile_waitroom.rs   # TurnstileWaitRoom
│   └── image_to_text.rs        # ImageToText
├── providers/
│   ├── mod.rs                  # Re-exports
│   ├── traits.rs               # Provider trait, TaskCreationOutcome
│   ├── retryable/              # CaptchaRetryableProvider wrapper
│   ├── capsolver/              # Capsolver implementation
│   │   ├── mod.rs
│   │   ├── provider.rs         # CapsolverProvider + Provider impl
│   │   ├── types.rs            # CapsolverTask enum + TryFrom + CapsolverSolution
│   │   ├── errors.rs           # CapsolverError + RetryableError impl
│   │   ├── response.rs         # API response parsing
│   │   └── tests.rs            # wiremock-based tests
│   ├── capmonster/             # (same structure)
│   └── rucaptcha/              # (same structure)
├── service/
│   ├── structure.rs            # CaptchaSolverService + polling loop
│   ├── traits.rs               # CaptchaSolverServiceTrait
│   ├── config.rs               # Config + presets (fast/balanced/patient)
│   ├── errors.rs               # ServiceError
│   └── tests.rs                # MockProvider-based tests
└── utils/
    ├── proxy.rs                # ProxyConfig, ApiProxyFields, RucaptchaProxyFields
    ├── retry.rs                # RetryConfig (backon wrapper)
    ├── types.rs                # TaskId newtype
    └── serde_helpers.rs        # String/number deserialization helpers
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `capsolver` | Yes | Capsolver provider |
| `capmonster` | Yes | CapMonster Cloud provider |
| `rucaptcha` | Yes | RuCaptcha provider |
| `tracing` | Yes | OpenTelemetry tracing instrumentation |
| `metrics` | No | OpenTelemetry metrics (counters, histograms) |
| `native-tls` | Yes | System TLS backend |
| `rustls-tls` | No | Rustls TLS backend |

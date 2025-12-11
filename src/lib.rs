//! # Captcha Solvers
//!
//! A generic captcha solving library with provider abstraction.
//!
//! This library provides a unified interface for working with different captcha
//! solving services. Each provider defines its own task and solution types,
//! enabling type-safe interactions.
//!
//! ## Architecture
//!
//! The library follows a layered architecture:
//!
//! ```text
//! CaptchaSolverService<P>
//!         |
//!         v
//! RetryableProvider<P>  (optional retry wrapper)
//!         |
//!         v
//!     Provider          (trait implemented by each provider)
//!         |
//!         v
//!   Provider Client     (HTTP client for the specific service)
//! ```
//!
//! ## Features
//!
//! - `capsolver` - Capsolver provider support (enabled by default)
//! - `tracing` - OpenTelemetry tracing instrumentation (enabled by default)
//!
//! ## Example
//!
//! ```rust,ignore
//! use captcha_solvers::{
//!     CaptchaSolverService, CaptchaSolverServiceConfig,
//!     CaptchaSolverServiceTrait, RetryableProvider,
//!     providers::capsolver::{CapsolverClient, CapsolverProvider, CapsolverTask},
//! };
//! use std::time::Duration;
//! use url::Url;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create the client
//!     let url = Url::parse("https://api.capsolver.com")?;
//!     let client = CapsolverClient::new(url, "your_api_key")?;
//!
//!     // Create the provider with retry support
//!     let provider = CapsolverProvider::new(client);
//!     let retryable = RetryableProvider::new(provider);
//!
//!     // Create the service
//!     let config = CaptchaSolverServiceConfig {
//!         poll_interval: Duration::from_secs(3),
//!     };
//!     let service = CaptchaSolverService::new(retryable, config);
//!
//!     // Solve a Turnstile captcha
//!     let task = CapsolverTask::turnstile(
//!         "https://example.com",
//!         "site_key_here",
//!     );
//!
//!     let solution = service.solve_captcha(task, Duration::from_secs(120)).await?;
//!     let turnstile = solution.into_turnstile();
//!     println!("Token: {}", turnstile.token);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Implementing a Custom Provider
//!
//! To add support for a new captcha service, implement the `Provider` trait:
//!
//! ```rust,ignore
//! use captcha_solvers::{Provider, TaskId, RetryableError};
//!
//! #[derive(Clone)]
//! struct MyProvider { /* ... */ }
//!
//! #[derive(Clone)]
//! enum MyTask {
//!     Turnstile { url: String, key: String },
//!     ReCaptcha { url: String, key: String },
//! }
//!
//! #[derive(Debug)]
//! struct MySolution { token: String }
//!
//! #[derive(Debug)]
//! enum MyError { /* ... */ }
//!
//! impl std::fmt::Display for MyTask {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         match self {
//!             Self::Turnstile { .. } => write!(f, "Turnstile"),
//!             Self::ReCaptcha { .. } => write!(f, "ReCaptcha"),
//!         }
//!     }
//! }
//!
//! impl RetryableError for MyError {
//!     fn is_retryable(&self) -> bool { false }
//! }
//!
//! impl Provider for MyProvider {
//!     type Task = MyTask;
//!     type Solution = MySolution;
//!     type Error = MyError;
//!
//!     async fn create_task(&self, task: Self::Task) -> Result<TaskId, Self::Error> {
//!         // Implementation
//!         todo!()
//!     }
//!
//!     async fn get_task_result(&self, task_id: &TaskId) -> Result<Option<Self::Solution>, Self::Error> {
//!         // Implementation
//!         todo!()
//!     }
//! }
//! ```

pub mod errors;
pub mod provider;
pub mod providers;
pub mod retry;
pub mod service;
pub mod types;

// Re-export commonly used types at the crate root
pub use errors::RetryableError;
pub use provider::{Provider, RetryableProvider};
pub use service::{CaptchaSolverService, CaptchaSolverServiceConfig, CaptchaSolverServiceTrait, ServiceError};
pub use types::TaskId;
//! Service-level lifecycle tests using a mock provider.

use crate::errors::RetryableError;
use crate::providers::traits::{Provider, TaskCreationOutcome};
use crate::service::{CaptchaSolverService, CaptchaSolverServiceConfig, CaptchaSolverServiceTrait};
use crate::solutions::ProviderSolution;
use crate::tasks::CaptchaTask;
use crate::utils::types::TaskId;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

// ============================================================================
// Mock provider infrastructure
// ============================================================================

#[derive(Debug, Clone)]
struct MockSolution {
    token: String,
}

impl ProviderSolution for MockSolution {}

#[derive(Debug, Error)]
enum MockError {
    #[error("mock transient error")]
    Transient,
    #[error("mock permanent error")]
    Permanent,
}

impl RetryableError for MockError {
    fn is_retryable(&self) -> bool {
        matches!(self, MockError::Transient)
    }

    fn should_retry_operation(&self) -> bool {
        matches!(self, MockError::Transient)
    }
}

/// What the mock provider should do on create_task.
#[derive(Clone)]
enum CreateBehavior {
    Pending,
    Ready,
    Error(bool), // true = transient, false = permanent
}

/// What the mock provider should do on get_task_result polls.
#[derive(Clone)]
enum PollBehavior {
    /// Return solution after N polls returning None.
    SuccessAfter(u32),
    /// Return transient error on poll N (0-indexed), then succeed on poll N+1.
    TransientThenSuccess {
        error_on_poll: u32,
        success_on_poll: u32,
    },
    /// Always return permanent error.
    PermanentError,
    /// Never return a solution (for timeout/cancel tests).
    NeverReady,
}

#[derive(Clone)]
struct MockProvider {
    create_behavior: CreateBehavior,
    poll_behavior: PollBehavior,
    poll_count: Arc<AtomicU32>,
}

impl MockProvider {
    fn new(create_behavior: CreateBehavior, poll_behavior: PollBehavior) -> Self {
        Self {
            create_behavior,
            poll_behavior,
            poll_count: Arc::new(AtomicU32::new(0)),
        }
    }
}

impl Provider for MockProvider {
    type Solution = MockSolution;
    type Error = MockError;

    async fn create_task(
        &self,
        _task: CaptchaTask,
    ) -> Result<TaskCreationOutcome<Self::Solution>, Self::Error> {
        match &self.create_behavior {
            CreateBehavior::Pending => {
                Ok(TaskCreationOutcome::Pending(TaskId::from("mock-task-123")))
            }
            CreateBehavior::Ready => Ok(TaskCreationOutcome::Ready {
                task_id: TaskId::from("mock-task-123"),
                solution: MockSolution {
                    token: "immediate-token".into(),
                },
            }),
            CreateBehavior::Error(transient) => {
                if *transient {
                    Err(MockError::Transient)
                } else {
                    Err(MockError::Permanent)
                }
            }
        }
    }

    async fn get_task_result(
        &self,
        _task_id: &TaskId,
    ) -> Result<Option<Self::Solution>, Self::Error> {
        let current = self.poll_count.fetch_add(1, Ordering::SeqCst);
        match &self.poll_behavior {
            PollBehavior::SuccessAfter(n) => {
                if current >= *n {
                    Ok(Some(MockSolution {
                        token: "polled-token".into(),
                    }))
                } else {
                    Ok(None)
                }
            }
            PollBehavior::TransientThenSuccess {
                error_on_poll,
                success_on_poll,
            } => {
                if current == *error_on_poll {
                    Err(MockError::Transient)
                } else if current >= *success_on_poll {
                    Ok(Some(MockSolution {
                        token: "recovered-token".into(),
                    }))
                } else {
                    Ok(None)
                }
            }
            PollBehavior::PermanentError => Err(MockError::Permanent),
            PollBehavior::NeverReady => Ok(None),
        }
    }
}

fn fast_config() -> CaptchaSolverServiceConfig {
    CaptchaSolverServiceConfig::builder()
        .timeout(Duration::from_secs(2))
        .poll_interval(Duration::from_millis(50))
        .build()
}

fn task() -> crate::tasks::Turnstile {
    crate::tasks::Turnstile::new("https://example.com", "site-key")
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_immediate_success() {
    let provider = MockProvider::new(CreateBehavior::Ready, PollBehavior::NeverReady);
    let service = CaptchaSolverService::with_config(provider, fast_config());

    let solution = service.solve_captcha(task()).await.unwrap();
    assert_eq!(solution.token, "immediate-token");
}

#[tokio::test]
async fn test_multi_poll_success() {
    let provider = MockProvider::new(CreateBehavior::Pending, PollBehavior::SuccessAfter(3));
    let service = CaptchaSolverService::with_config(provider, fast_config());

    let solution = service.solve_captcha(task()).await.unwrap();
    assert_eq!(solution.token, "polled-token");
}

#[tokio::test]
async fn test_transient_error_then_success() {
    let provider = MockProvider::new(
        CreateBehavior::Pending,
        PollBehavior::TransientThenSuccess {
            error_on_poll: 1,
            success_on_poll: 3,
        },
    );
    let service = CaptchaSolverService::with_config(provider, fast_config());

    let solution = service.solve_captcha(task()).await.unwrap();
    assert_eq!(solution.token, "recovered-token");
}

#[tokio::test]
async fn test_permanent_provider_error_on_create() {
    let provider = MockProvider::new(CreateBehavior::Error(false), PollBehavior::NeverReady);
    let service = CaptchaSolverService::with_config(provider, fast_config());

    let err = service.solve_captcha(task()).await.unwrap_err();
    assert!(!err.is_retryable());
    assert!(!err.is_timeout());
    assert!(!err.is_cancelled());
}

#[tokio::test]
async fn test_permanent_provider_error_on_poll() {
    let provider = MockProvider::new(CreateBehavior::Pending, PollBehavior::PermanentError);
    let service = CaptchaSolverService::with_config(provider, fast_config());

    let err = service.solve_captcha(task()).await.unwrap_err();
    assert!(!err.is_retryable());
    assert!(!err.is_timeout());
}

#[tokio::test]
async fn test_timeout() {
    let config = CaptchaSolverServiceConfig::builder()
        .timeout(Duration::from_millis(200))
        .poll_interval(Duration::from_millis(50))
        .build();

    let provider = MockProvider::new(CreateBehavior::Pending, PollBehavior::NeverReady);
    let service = CaptchaSolverService::with_config(provider, config);

    let err = service.solve_captcha(task()).await.unwrap_err();
    assert!(err.is_timeout());
    assert!(err.elapsed().unwrap() >= Duration::from_millis(200));
    assert!(err.poll_count().unwrap() > 0);
}

#[tokio::test]
async fn test_cancellation() {
    let config = CaptchaSolverServiceConfig::builder()
        .timeout(Duration::from_secs(10))
        .poll_interval(Duration::from_millis(50))
        .build();

    let provider = MockProvider::new(CreateBehavior::Pending, PollBehavior::NeverReady);
    let service = CaptchaSolverService::with_config(provider, config);

    let cancel_token = CancellationToken::new();
    let token_clone = cancel_token.clone();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(150)).await;
        token_clone.cancel();
    });

    let err = service
        .solve_captcha_cancellable(task(), cancel_token)
        .await
        .unwrap_err();
    assert!(err.is_cancelled());
    assert!(err.poll_count().unwrap() > 0);
}

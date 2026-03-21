use crate::errors::RetryableError;
use opentelemetry::trace::Status;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use super::error_chain::ErrorChain;

#[inline]
pub(crate) fn set_span_ok() {
    Span::current().set_status(Status::Ok);
}

#[inline]
pub(crate) fn set_span_error(err: &dyn std::fmt::Display) {
    Span::current().set_status(Status::error(err.to_string()));
}

/// Record error with appropriate span status and log level.
///
/// Transient errors get a warning log and don't set error status.
/// Permanent errors set error span status and log at error level.
#[inline]
pub(crate) fn record_error<E: std::error::Error + RetryableError>(e: &E, provider: &'static str) {
    if e.is_retryable() {
        tracing::warn!(error = %ErrorChain(e), "{provider} transient error");
    } else {
        set_span_error(&ErrorChain(e));
        tracing::error!(error = %ErrorChain(e), "{provider} operation failed");
    }
}

//! Response parsing for the Capsolver API.

use super::errors::CapsolverApiError;
use crate::impl_api_response_deserialize;

/// Capsolver API response wrapper
#[derive(Debug)]
pub enum CapsolverResponse<T> {
    Success(T),
    Error(CapsolverApiError),
}

impl<T> CapsolverResponse<T> {
    /// Convert to Result for convenient use with ?
    pub fn into_result(self) -> Result<T, CapsolverApiError> {
        match self {
            Self::Success(data) => Ok(data),
            Self::Error(e) => Err(e),
        }
    }
}

impl_api_response_deserialize!(CapsolverResponse, CapsolverApiError);
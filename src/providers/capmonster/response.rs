//! Response parsing for the CapMonster API.

use super::errors::CapmonsterApiError;
use crate::impl_api_response_deserialize;

/// CapMonster API response wrapper
#[derive(Debug)]
pub enum CapmonsterResponse<T> {
    Success(T),
    Error(CapmonsterApiError),
}

impl<T> CapmonsterResponse<T> {
    /// Convert to Result for convenient use with `?`.
    pub fn into_result(self) -> Result<T, CapmonsterApiError> {
        match self {
            Self::Success(data) => Ok(data),
            Self::Error(err) => Err(err),
        }
    }
}

impl_api_response_deserialize!(CapmonsterResponse, CapmonsterApiError);

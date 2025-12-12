//! Response parsing for the RuCaptcha API.

use super::errors::RucaptchaApiError;
use crate::impl_api_response_deserialize;

/// RuCaptcha API response wrapper
#[derive(Debug)]
pub enum RucaptchaResponse<T> {
    Success(T),
    Error(RucaptchaApiError),
}

impl<T> RucaptchaResponse<T> {
    /// Convert to Result for convenient use with ?
    pub fn into_result(self) -> Result<T, RucaptchaApiError> {
        match self {
            Self::Success(data) => Ok(data),
            Self::Error(e) => Err(e),
        }
    }
}

impl_api_response_deserialize!(RucaptchaResponse, RucaptchaApiError);

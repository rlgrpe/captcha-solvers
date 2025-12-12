//! Response parsing for the RuCaptcha API.

use super::errors::RucaptchaApiError;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

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

impl<'de, T> Deserialize<'de> for RucaptchaResponse<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json_value: Value = Deserialize::deserialize(deserializer)?;

        let error_id = json_value
            .get("errorId")
            .and_then(Value::as_u64)
            .unwrap_or(0);

        if error_id != 0 {
            let api_error: RucaptchaApiError =
                serde_json::from_value(json_value).map_err(serde::de::Error::custom)?;
            return Ok(RucaptchaResponse::Error(api_error));
        }

        serde_json::from_value::<T>(json_value)
            .map(RucaptchaResponse::Success)
            .map_err(serde::de::Error::custom)
    }
}
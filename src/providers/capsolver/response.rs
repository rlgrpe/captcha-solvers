use super::errors::CapsolverApiError;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

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

    /// Check without consuming
    #[allow(dead_code)]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Reference to data if success
    #[allow(dead_code)]
    pub fn as_success(&self) -> Option<&T> {
        match self {
            Self::Success(data) => Some(data),
            Self::Error(_) => None,
        }
    }
}

impl<'de, T> Deserialize<'de> for CapsolverResponse<T>
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
            let api_error: CapsolverApiError =
                serde_json::from_value(json_value).map_err(serde::de::Error::custom)?;
            return Ok(CapsolverResponse::Error(api_error));
        }

        serde_json::from_value::<T>(json_value)
            .map(CapsolverResponse::Success)
            .map_err(serde::de::Error::custom)
    }
}
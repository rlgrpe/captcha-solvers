//! Generic response handling for captcha solver APIs.
//!
//! This module provides a unified response parsing mechanism that works
//! across different captcha solving providers.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::fmt::Debug;

/// Generic API response wrapper
///
/// This enum represents the two possible outcomes of an API call:
/// - Success with data of type `T`
/// - Error with provider-specific error type `E`
#[derive(Debug)]
pub enum ApiResponse<T, E> {
    /// Successful response with data
    Success(T),
    /// Error response with provider-specific error
    Error(E),
}

impl<T, E> ApiResponse<T, E> {
    /// Convert to Result for convenient use with ?
    pub fn into_result(self) -> Result<T, E> {
        match self {
            Self::Success(data) => Ok(data),
            Self::Error(e) => Err(e),
        }
    }

    /// Check if this is a success response
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Check if this is an error response
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}

/// Deserialize an API response that uses errorId field to indicate errors
///
/// This function handles the common pattern where:
/// - `errorId == 0` indicates success
/// - `errorId != 0` indicates an error
///
/// # Type Parameters
///
/// * `T` - The success data type
/// * `E` - The error type (must be deserializable from the JSON response)
pub fn deserialize_error_id_response<'de, D, T, E>(
    deserializer: D,
) -> Result<ApiResponse<T, E>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
    E: DeserializeOwned,
{
    let json_value: Value = Deserialize::deserialize(deserializer)?;

    let error_id = json_value
        .get("errorId")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    if error_id != 0 {
        let api_error: E = serde_json::from_value(json_value).map_err(serde::de::Error::custom)?;
        return Ok(ApiResponse::Error(api_error));
    }

    serde_json::from_value::<T>(json_value)
        .map(ApiResponse::Success)
        .map_err(serde::de::Error::custom)
}

/// Macro to implement Deserialize for provider-specific response types
///
/// This reduces boilerplate for implementing the standard error-id based
/// response deserialization pattern.
#[macro_export]
macro_rules! impl_api_response_deserialize {
    ($response_type:ident, $error_type:ty) => {
        impl<'de, T> serde::Deserialize<'de> for $response_type<T>
        where
            T: serde::de::DeserializeOwned,
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let response =
                    $crate::utils::response::deserialize_error_id_response::<D, T, $error_type>(
                        deserializer,
                    )?;
                Ok(match response {
                    $crate::utils::response::ApiResponse::Success(data) => Self::Success(data),
                    $crate::utils::response::ApiResponse::Error(err) => Self::Error(err),
                })
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde::de::IntoDeserializer;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestData {
        value: String,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    struct TestError {
        error_id: u64,
        error_code: String,
    }

    #[test]
    fn test_api_response_success() {
        let json = r#"{"errorId": 0, "value": "test"}"#;
        let value: Value = serde_json::from_str(json).unwrap();
        let response: ApiResponse<TestData, TestError> =
            deserialize_error_id_response(value.into_deserializer()).unwrap();

        assert!(response.is_success());
        let result = response.into_result().unwrap();
        assert_eq!(result.value, "test");
    }

    #[test]
    fn test_api_response_error() {
        let json = r#"{"errorId": 1, "errorCode": "ERROR_TEST"}"#;
        let value: Value = serde_json::from_str(json).unwrap();
        let response: ApiResponse<TestData, TestError> =
            deserialize_error_id_response(value.into_deserializer()).unwrap();

        assert!(response.is_error());
        let err = response.into_result().unwrap_err();
        assert_eq!(err.error_id, 1);
        assert_eq!(err.error_code, "ERROR_TEST");
    }
}

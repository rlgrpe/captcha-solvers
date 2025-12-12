//! Shared serialization and deserialization helpers.
//!
//! This module provides common serde utilities used across different
//! captcha solving providers.

use serde::{Deserialize, Deserializer, Serializer};

/// Skip serializing if the value is false
pub fn skip_if_false(value: &bool) -> bool {
    !*value
}

/// Skip serializing if the Option is None
pub fn skip_if_none<T>(value: &Option<T>) -> bool {
    value.is_none()
}

/// Deserialize a value that can be either a string or a number into a String
///
/// This is useful for APIs that inconsistently return numeric IDs as either
/// strings or numbers.
pub fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        _ => Err(D::Error::custom("expected string or number")),
    }
}

/// Serialize a string as a number if it's numeric, otherwise as a string
///
/// This is useful for APIs that expect numeric task IDs but our internal
/// representation stores them as strings.
pub fn serialize_string_as_number_if_possible<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Try to parse as u64 first (most common for task IDs)
    if let Ok(n) = value.parse::<u64>() {
        return serializer.serialize_u64(n);
    }
    // Fall back to string
    serializer.serialize_str(value)
}

/// Module for optional boolean serialization
pub mod optional_bool {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => v.serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<bool>::deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        #[serde(deserialize_with = "deserialize_string_or_number")]
        id: String,
    }

    #[test]
    fn test_deserialize_string_or_number_with_string() {
        let json = json!({"id": "12345"});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.id, "12345");
    }

    #[test]
    fn test_deserialize_string_or_number_with_number() {
        let json = json!({"id": 12345});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.id, "12345");
    }

    #[derive(Serialize)]
    struct TestSerialize<'a> {
        #[serde(serialize_with = "serialize_string_as_number_if_possible")]
        id: &'a str,
    }

    #[test]
    fn test_serialize_string_as_number_numeric() {
        let test = TestSerialize { id: "12345" };
        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, r#"{"id":12345}"#);
    }

    #[test]
    fn test_serialize_string_as_number_non_numeric() {
        let test = TestSerialize { id: "abc-123" };
        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, r#"{"id":"abc-123"}"#);
    }

    #[test]
    fn test_skip_if_false() {
        assert!(!skip_if_false(&true));
        assert!(skip_if_false(&false));
    }

    #[test]
    fn test_skip_if_none() {
        assert!(skip_if_none(&None::<i32>));
        assert!(!skip_if_none(&Some(42)));
    }
}
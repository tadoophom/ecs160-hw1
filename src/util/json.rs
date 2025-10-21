//! JSON parsing utilities used by model builders to keep conversion logic shared.
//! Consolidates boilerplate for extracting typed values from `serde_json::Value`.
use serde_json::{Map, Value};

use crate::error::AppError;

pub fn json_error(message: impl Into<String>) -> AppError {
    AppError::Serialization(<serde_json::Error as serde::de::Error>::custom(
        message.into(),
    ))
}

pub fn as_object<'a>(value: &'a Value, context: &str) -> Result<&'a Map<String, Value>, AppError> {
    value
        .as_object()
        .ok_or_else(|| json_error(format!("{context} expected to be a JSON object")))
}

pub fn required_field<'a>(map: &'a Map<String, Value>, field: &str) -> Result<&'a Value, AppError> {
    map.get(field)
        .ok_or_else(|| json_error(format!("missing `{field}` field")))
}

pub fn required_string(map: &Map<String, Value>, field: &str) -> Result<String, AppError> {
    required_field(map, field)?
        .as_str()
        .map(|value| value.to_string())
        .ok_or_else(|| json_error(format!("`{field}` must be a string")))
}

pub fn required_bool(map: &Map<String, Value>, field: &str) -> Result<bool, AppError> {
    required_field(map, field)?
        .as_bool()
        .ok_or_else(|| json_error(format!("`{field}` must be a boolean")))
}

pub fn required_i64(map: &Map<String, Value>, field: &str) -> Result<i64, AppError> {
    required_field(map, field)?
        .as_i64()
        .ok_or_else(|| json_error(format!("`{field}` must be a 64-bit integer")))
}

pub fn optional_string(map: &Map<String, Value>, field: &str) -> Option<String> {
    map.get(field)
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}

pub fn optional_u64(map: &Map<String, Value>, field: &str) -> u64 {
    map.get(field)
        .and_then(|value| value.as_u64())
        .unwrap_or_default()
}

pub fn optional_i64(map: &Map<String, Value>, field: &str) -> i64 {
    map.get(field)
        .and_then(|value| value.as_i64())
        .unwrap_or_default()
}

pub fn parse_optional<T, F>(
    map: &Map<String, Value>,
    field: &str,
    parser: F,
) -> Result<Option<T>, AppError>
where
    F: Fn(&Value) -> Result<T, AppError>,
{
    match map.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => parser(value).map(Some),
    }
}

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

// Generic extractor for required values with type conversion
fn extract_required<T, F>(
    map: &Map<String, Value>,
    field: &str,
    extractor: F,
) -> Result<T, AppError>
where
    F: Fn(&Value) -> Option<T>,
{
    required_field(map, field)
        .and_then(|v| extractor(v).ok_or_else(|| json_error(format!("`{field}` has invalid type"))))
}

fn extract_optional<T, F>(map: &Map<String, Value>, field: &str, extractor: F) -> Option<T>
where
    F: Fn(&Value) -> Option<T>,
{
    map.get(field).and_then(extractor)
}

pub fn required_string(map: &Map<String, Value>, field: &str) -> Result<String, AppError> {
    extract_required(map, field, |v| v.as_str().map(|s| s.to_string()))
}

pub fn required_bool(map: &Map<String, Value>, field: &str) -> Result<bool, AppError> {
    extract_required(map, field, |v| v.as_bool())
}

pub fn required_i64(map: &Map<String, Value>, field: &str) -> Result<i64, AppError> {
    extract_required(map, field, |v| v.as_i64())
}

pub fn optional_string(map: &Map<String, Value>, field: &str) -> Option<String> {
    extract_optional(map, field, |v| v.as_str().map(|s| s.to_string()))
}

pub fn optional_u64(map: &Map<String, Value>, field: &str) -> u64 {
    extract_optional(map, field, |v| v.as_u64()).unwrap_or_default()
}

pub fn optional_i64(map: &Map<String, Value>, field: &str) -> i64 {
    extract_optional(map, field, |v| v.as_i64()).unwrap_or_default()
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
        Some(value) => Ok(Some(parser(value)?)),
        None => Ok(None),
    }
}

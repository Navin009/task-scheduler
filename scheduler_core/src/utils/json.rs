// src/utils/json.rs
use crate::error::{Result, SchedulerError};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Value};

/// Serializes a Rust type into a JSON string.
pub fn to_json_string<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(SchedulerError::Json)
}

/// Deserializes a JSON string into a Rust type.
pub fn from_json_string<'a, T: Deserialize<'a>>(s: &'a str) -> Result<T> {
    serde_json::from_str(s).map_err(SchedulerError::Json)
}

/// Serializes a Rust type into a `serde_json::Value`.
pub fn to_json_value<T: Serialize>(value: &T) -> Result<Value> {
    serde_json::to_value(value).map_err(SchedulerError::Json)
}

/// Deserializes a `serde_json::Value` into a Rust type.
pub fn from_json_value<T: DeserializeOwned>(value: Value) -> Result<T> {
    serde_json::from_value(value).map_err(SchedulerError::Json)
}

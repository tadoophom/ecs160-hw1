//! Models GitHub issues and provides JSON deserialization helpers.
//! Captures basic metadata needed for analytics and storage tasks.
use serde_json::Value;

use crate::error::AppError;
use crate::util::json::{as_object, optional_string, required_string};

#[derive(Debug, Clone)]
pub struct Issue {
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Issue {
    pub fn from_json(value: &Value) -> Result<Self, AppError> {
        let map = as_object(value, "issue")?;

        Ok(Self {
            title: required_string(map, "title")?,
            body: optional_string(map, "body"),
            state: required_string(map, "state")?,
            html_url: optional_string(map, "html_url"),
            created_at: required_string(map, "created_at")?,
            updated_at: required_string(map, "updated_at")?,
        })
    }
}

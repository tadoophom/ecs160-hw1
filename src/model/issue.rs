//! Issue model.
use serde_json::Value;

use crate::error::AppError;
use crate::util::json::{as_object, optional_string, required_string, required_i64};

#[derive(Debug, Clone)]
pub struct Issue {
    pub id: i64,
    pub number: i64,
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
            id: required_i64(map, "id")?,
            number: required_i64(map, "number")?,
            title: required_string(map, "title")?,
            body: optional_string(map, "body"),
            state: required_string(map, "state")?,
            html_url: optional_string(map, "html_url"),
            created_at: required_string(map, "created_at")?,
            updated_at: required_string(map, "updated_at")?,
        })
    }
}

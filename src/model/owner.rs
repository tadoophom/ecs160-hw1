//! Represents repository owners and extracts identity fields from GitHub JSON.
//! Keeps ownership metadata reusable throughout the model layer.
use serde_json::Value;

use crate::error::AppError;
use crate::util::json::{as_object, required_bool, required_i64, required_string};

#[derive(Debug, Clone)]
pub struct Owner {
    pub login: String,
    pub id: i64,
    pub html_url: String,
    pub site_admin: bool,
}

impl Owner {
    pub fn from_json(value: &Value) -> Result<Self, AppError> {
        let map = as_object(value, "owner")?;

        Ok(Self {
            login: required_string(map, "login")?,
            id: required_i64(map, "id")?,
            html_url: required_string(map, "html_url")?,
            site_admin: required_bool(map, "site_admin")?,
        })
    }
}

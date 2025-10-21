//! Represents Git commit payloads and related JSON parsing helpers.
//! Tracks summary metadata plus file-level change details for analytics.
use serde_json::Value;

use crate::error::AppError;
use crate::util::json::{
    as_object, optional_i64, optional_string, parse_optional, required_field, required_string,
};

#[derive(Debug, Clone)]
pub struct Commit {
    pub sha: String,
    pub url: String,
    pub html_url: Option<String>,
    pub commit: CommitSummary,
    pub files: Vec<CommitFile>,
}

impl Commit {
    pub fn from_json(value: &Value) -> Result<Self, AppError> {
        let map = as_object(value, "commit")?;

        Ok(Self {
            sha: required_string(map, "sha")?,
            url: optional_string(map, "url").unwrap_or_default(),
            html_url: optional_string(map, "html_url"),
            commit: CommitSummary::from_json(required_field(map, "commit")?)?,
            files: match map.get("files") {
                Some(Value::Array(items)) => items
                    .iter()
                    .map(CommitFile::from_json)
                    .collect::<Result<Vec<_>, _>>()?,
                _ => Vec::new(),
            },
        })
    }
}

#[derive(Debug, Clone)]
pub struct CommitSummary {
    pub message: String,
    pub author: Option<CommitAuthor>,
    pub committer: Option<CommitAuthor>,
}

impl CommitSummary {
    pub fn from_json(value: &Value) -> Result<Self, AppError> {
        let map = as_object(value, "commit summary")?;

        Ok(Self {
            message: required_string(map, "message")?,
            author: parse_optional(map, "author", CommitAuthor::from_json)?,
            committer: parse_optional(map, "committer", CommitAuthor::from_json)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CommitAuthor {
    pub name: Option<String>,
    pub email: Option<String>,
    pub date: Option<String>,
}

impl CommitAuthor {
    pub fn from_json(value: &Value) -> Result<Self, AppError> {
        let map = as_object(value, "commit author")?;

        Ok(Self {
            name: optional_string(map, "name"),
            email: optional_string(map, "email"),
            date: optional_string(map, "date"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CommitFile {
    pub filename: String,
    pub additions: i64,
    pub deletions: i64,
    pub changes: i64,
    pub status: String,
}

impl CommitFile {
    pub fn from_json(value: &Value) -> Result<Self, AppError> {
        let map = as_object(value, "commit file")?;

        Ok(Self {
            filename: required_string(map, "filename")?,
            additions: optional_i64(map, "additions"),
            deletions: optional_i64(map, "deletions"),
            changes: optional_i64(map, "changes"),
            status: optional_string(map, "status").unwrap_or_default(),
        })
    }
}

use serde::{Deserialize, Serialize};
use crate::error::AppError;
use serde_json::{Map, Value};

fn json_error(message: impl Into<String>) -> AppError {
    AppError::Serialization(<serde_json::Error as serde::de::Error>::custom(
        message.into(),
    ))
}

fn as_object<'a>(value: &'a Value, context: &str) -> Result<&'a Map<String, Value>, AppError> {
    value
        .as_object()
        .ok_or_else(|| json_error(format!("{context} expected to be a JSON object")))
}

fn required_field<'a>(map: &'a Map<String, Value>, field: &str) -> Result<&'a Value, AppError> {
    map.get(field)
        .ok_or_else(|| json_error(format!("missing `{field}` field")))
}

fn required_string(map: &Map<String, Value>, field: &str) -> Result<String, AppError> {
    required_field(map, field)?
        .as_str()
        .map(|value| value.to_string())
        .ok_or_else(|| json_error(format!("`{field}` must be a string")))
}

fn required_bool(map: &Map<String, Value>, field: &str) -> Result<bool, AppError> {
    required_field(map, field)?
        .as_bool()
        .ok_or_else(|| json_error(format!("`{field}` must be a boolean")))
}

fn required_i64(map: &Map<String, Value>, field: &str) -> Result<i64, AppError> {
    required_field(map, field)?
        .as_i64()
        .ok_or_else(|| json_error(format!("`{field}` must be a 64-bit integer")))
}

fn optional_string(map: &Map<String, Value>, field: &str) -> Option<String> {
    map.get(field)
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}

fn optional_u64(map: &Map<String, Value>, field: &str) -> u64 {
    map.get(field)
        .and_then(|value| value.as_u64())
        .unwrap_or_default()
}

fn optional_i64(map: &Map<String, Value>, field: &str) -> i64 {
    map.get(field)
        .and_then(|value| value.as_i64())
        .unwrap_or_default()
}

fn parse_optional<T, F>(
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub login: String,
    pub id: i64,
    #[serde(rename = "html_url")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub id: i64,
    pub name: String,
    #[serde(rename = "full_name")]
    pub full_name: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    #[serde(rename = "forks_count", default)]
    pub forks_count: u64,
    #[serde(rename = "stargazers_count", default)]
    pub stargazers_count: u64,
    #[serde(rename = "open_issues_count", default)]
    pub open_issues_count: u64,
    pub language: Option<String>,
    pub owner: Owner,
    #[serde(skip)]
    pub forks: Vec<Repo>,
    #[serde(skip)]
    pub recent_commits: Vec<Commit>,
    #[serde(skip)]
    pub issues: Vec<Issue>,
    #[serde(skip)]
    pub commit_count: u64,
}

impl Repo {
    pub fn slug(&self) -> String {
        format!("{}/{}", self.owner.login, self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Commit {
    pub sha: String,
    #[serde(default)]
    pub url: String,
    #[serde(rename = "html_url", default)]
    pub html_url: Option<String>,
    pub commit: CommitSummary,
    #[serde(default)]
    pub files: Vec<CommitFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommitSummary {
    pub message: String,
    pub author: Option<CommitAuthor>,
    pub committer: Option<CommitAuthor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommitAuthor {
    pub name: Option<String>,
    pub email: Option<String>,
    #[serde(rename = "date", default)]
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommitFile {
    pub filename: String,
    #[serde(default)]
    pub additions: i64,
    #[serde(default)]
    pub deletions: i64,
    #[serde(default)]
    pub changes: i64,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub title: String,
    #[serde(default)]
    pub body: Option<String>,
    pub state: String,
    #[serde(rename = "html_url", default)]
    pub html_url: Option<String>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
}

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

#[derive(Debug, Clone)]
pub struct Repo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub forks_count: u64,
    pub stargazers_count: u64,
    pub open_issues_count: u64,
    pub language: Option<String>,
    pub owner: Owner,
    pub forks: Vec<Repo>,
    pub recent_commits: Vec<Commit>,
    pub issues: Vec<Issue>,
    pub commit_count: u64,
}

impl Repo {
    pub fn from_json(value: &Value) -> Result<Self, AppError> {
        let map = as_object(value, "repository")?;

        Ok(Self {
            id: required_i64(map, "id")?,
            name: required_string(map, "name")?,
            full_name: required_string(map, "full_name")?,
            html_url: required_string(map, "html_url")?,
            forks_count: optional_u64(map, "forks_count"),
            stargazers_count: optional_u64(map, "stargazers_count"),
            open_issues_count: optional_u64(map, "open_issues_count"),
            language: optional_string(map, "language"),
            owner: Owner::from_json(required_field(map, "owner")?)?,
            forks: Vec::new(),
            recent_commits: Vec::new(),
            issues: Vec::new(),
            commit_count: 0,
        })
    }

    pub fn slug(&self) -> String {
        format!("{}/{}", self.owner.login, self.name)
    }
}

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

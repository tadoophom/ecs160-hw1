//! Repository model.
use serde_json::Value;

use crate::error::AppError;
use crate::util::json::{
    as_object, optional_string, optional_u64, required_field, required_i64, required_string,
    optional_bool,
};

use super::{Commit, Issue, Owner};

#[derive(Debug, Clone)]
pub struct Repo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub forks_count: u64,
    pub stargazers_count: u64,
    pub open_issues_count: u64,
    pub has_issues: bool,
    pub language: Option<String>,
    pub owner: Owner,
    pub created_at: Option<String>,
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
            has_issues: optional_bool(map, "has_issues").unwrap_or(true),
            language: optional_string(map, "language"),
            owner: Owner::from_json(required_field(map, "owner")?)?,
            created_at: optional_string(map, "created_at"),
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

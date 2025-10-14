use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub login: String,
    pub id: i64,
    #[serde(rename = "html_url")]
    pub html_url: String,
    pub site_admin: bool,
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

//! Mock implementations demonstrating Liskov Substitution Principle compliance.
//! These implementations can be substituted for real services without breaking functionality.

use crate::error::AppError;
use crate::model::{Commit, Issue, Repo};
use crate::service::traits::{GitRepositoryService, DataStorageService};

/// Mock Git service for testing - can substitute GitService without breaking functionality
pub struct MockGitService {
    pub repos: Vec<Repo>,
    pub commits: Vec<Commit>,
    pub issues: Vec<Issue>,
}

impl MockGitService {
    pub fn new() -> Self {
        Self {
            repos: Vec::new(),
            commits: Vec::new(),
            issues: Vec::new(),
        }
    }

    pub fn with_repos(mut self, repos: Vec<Repo>) -> Self {
        self.repos = repos;
        self
    }

    pub fn with_commits(mut self, commits: Vec<Commit>) -> Self {
        self.commits = commits;
        self
    }

    pub fn with_issues(mut self, issues: Vec<Issue>) -> Self {
        self.issues = issues;
        self
    }
}

impl GitRepositoryService for MockGitService {
    async fn fetch_top_repositories(&self, _language: &str, per_page: u8) -> Result<Vec<Repo>, AppError> {
        Ok(self.repos.iter().take(per_page as usize).cloned().collect())
    }

    async fn fetch_repo_forks(&self, _owner: &str, _repo: &str) -> Result<Vec<Repo>, AppError> {
        Ok(Vec::new()) // Mock returns empty forks
    }

    async fn fetch_recent_commits(&self, _owner: &str, _repo: &str) -> Result<Vec<Commit>, AppError> {
        Ok(self.commits.clone())
    }

    async fn fetch_open_issues(&self, _owner: &str, _repo: &str) -> Result<Vec<Issue>, AppError> {
        Ok(self.issues.clone())
    }

    async fn fetch_commit_with_files(&self, _owner: &str, _repo: &str, _sha: &str) -> Result<Commit, AppError> {
        self.commits.first()
            .cloned()
            .ok_or_else(|| AppError::Config("No commits available".to_string()))
    }
}

/// Mock storage service for testing - can substitute RedisService without breaking functionality
pub struct MockStorageService {
    pub stored_repos: std::collections::HashMap<String, ()>,
}

impl MockStorageService {
    pub fn new() -> Self {
        Self {
            stored_repos: std::collections::HashMap::new(),
        }
    }
}

impl DataStorageService for MockStorageService {
    async fn store_repository(&mut self, repo: &Repo) -> Result<(), AppError> {
        let key = format!("{}:{}", repo.owner.login, repo.name);
        self.stored_repos.insert(key, ());
        Ok(())
    }
}

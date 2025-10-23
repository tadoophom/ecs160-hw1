//! Test implementations for development and testing.
//! These implementations can be substituted for real services without breaking functionality.

use crate::error::AppError;
use crate::model::{Commit, Issue, Repo};
use crate::service::traits::{GitRepositoryService, DataStorageService};

/// Test Git service for development and testing
pub struct TestGitService {
    pub repos: Vec<Repo>,
    pub commits: Vec<Commit>,
    pub issues: Vec<Issue>,
}

impl TestGitService {
    pub fn new() -> Self {
        Self {
            repos: Vec::new(),
            commits: Vec::new(),
            issues: Vec::new(),
        }
    }
}

impl GitRepositoryService for TestGitService {
    async fn fetch_top_repositories(&self, _language: &str, per_page: u8) -> Result<Vec<Repo>, AppError> {
        Ok(self.repos.iter().take(per_page as usize).cloned().collect())
    }

    async fn fetch_repo_forks(&self, _owner: &str, _repo: &str) -> Result<Vec<Repo>, AppError> {
        Ok(Vec::new()) // Test returns empty forks
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

/// Test storage service for development and testing
pub struct TestStorageService {
    pub stored_repos: std::collections::HashMap<String, ()>,
}

impl TestStorageService {
    pub fn new() -> Self {
        Self {
            stored_repos: std::collections::HashMap::new(),
        }
    }
}

impl DataStorageService for TestStorageService {
    async fn store_repository(&mut self, repo: &Repo) -> Result<(), AppError> {
        let key = format!("{}:{}", repo.owner.login, repo.name);
        self.stored_repos.insert(key, ());
        Ok(())
    }
}

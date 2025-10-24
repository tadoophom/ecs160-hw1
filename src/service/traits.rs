//! Service abstractions following Open/Closed Principle.
//! Defines interfaces that can be extended without modifying existing code.

use crate::error::AppError;
use crate::model::{Commit, Issue, Repo};

/// Abstract interface for Git repository services
/// Allows extension to different Git providers (GitHub, GitLab, etc.)
#[allow(async_fn_in_trait)]
pub trait GitRepositoryService {
    async fn fetch_top_repositories(
        &self,
        language: &str,
        per_page: u8,
    ) -> Result<Vec<Repo>, AppError>;
    async fn fetch_repo_forks(&self, owner: &str, repo: &str) -> Result<Vec<Repo>, AppError>;
    async fn fetch_recent_commits(&self, owner: &str, repo: &str) -> Result<Vec<Commit>, AppError>;
    async fn fetch_open_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>, AppError>;
    async fn fetch_commit_with_files(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
    ) -> Result<Commit, AppError>;
}

/// Abstract interface for data storage services
/// Allows extension to different storage backends (Redis, PostgreSQL, etc.)
#[allow(async_fn_in_trait)]
pub trait DataStorageService {
    async fn store_repository(&mut self, repo: &Repo) -> Result<(), AppError>;
}

/// Represents repository data retrieved from storage
#[derive(Debug, Clone)]
pub struct RepoData {
    pub url: String,
    pub name: String,
    pub owner: String,
    pub language: String,
    pub stars: u64,
    pub forks: u64,
    pub open_issues: u64,
}

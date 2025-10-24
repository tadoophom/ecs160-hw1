//! Service interfaces for different responsibilities.
//! Each interface focuses on a specific task to avoid forcing clients to depend on unused methods.

use crate::error::AppError;
use crate::model::{Commit, Issue, Repo};

/// Interface for basic repository information retrieval
pub trait RepoFetcher {
    async fn fetch_top_repositories(
        &self,
        language: &str,
        per_page: u8,
    ) -> Result<Vec<Repo>, AppError>;
    async fn fetch_repo_forks(&self, owner: &str, repo: &str) -> Result<Vec<Repo>, AppError>;
}

/// Interface for commit-related operations
pub trait CommitFetcher {
    async fn fetch_recent_commits(&self, owner: &str, repo: &str) -> Result<Vec<Commit>, AppError>;
    async fn fetch_commit_with_files(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
    ) -> Result<Commit, AppError>;
}

/// Interface for issue-related operations
pub trait IssueFetcher {
    async fn fetch_open_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>, AppError>;
}

/// Interface for basic repository storage operations
pub trait RepoStorage {
    async fn store_repository(&mut self, repo: &Repo) -> Result<(), AppError>;
}

/// Complete Git service interface combining all fetching capabilities
pub trait GitService: RepoFetcher + CommitFetcher + IssueFetcher {}

/// Complete storage service interface
pub trait StorageService: RepoStorage {}

// Note: Individual trait implementations would be added here for specific services
// For now, we'll use the simpler approach with the main traits module

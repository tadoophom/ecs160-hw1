//! Service interfaces for different responsibilities.
//! Each interface focuses on a specific task to avoid forcing clients to depend on unused methods.

use crate::error::AppError;
use crate::model::{Commit, Issue, Repo};

/// Interface for basic repository information retrieval
pub trait RepoFetcher {
    async fn fetch_top_repositories(&self, language: &str, per_page: u8) -> Result<Vec<Repo>, AppError>;
    async fn fetch_repo_forks(&self, owner: &str, repo: &str) -> Result<Vec<Repo>, AppError>;
}

/// Interface for commit-related operations
pub trait CommitFetcher {
    async fn fetch_recent_commits(&self, owner: &str, repo: &str) -> Result<Vec<Commit>, AppError>;
    async fn fetch_commit_with_files(&self, owner: &str, repo: &str, sha: &str) -> Result<Commit, AppError>;
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

/// Service for analyzing repository trends by language
pub struct LanguageAnalyzer<S: RepoFetcher> {
    fetcher: S,
}

impl<S: RepoFetcher> LanguageAnalyzer<S> {
    pub fn new(fetcher: S) -> Self {
        Self { fetcher }
    }

    pub async fn get_top_repos(&self, language: &str) -> Result<Vec<Repo>, AppError> {
        self.fetcher.fetch_top_repositories(language, 10).await
    }
}

/// Service for analyzing commit patterns
pub struct CommitAnalyzer<S: CommitFetcher> {
    fetcher: S,
}

impl<S: CommitFetcher> CommitAnalyzer<S> {
    pub fn new(fetcher: S) -> Self {
        Self { fetcher }
    }

    pub async fn get_recent_commits(&self, owner: &str, repo: &str) -> Result<Vec<Commit>, AppError> {
        self.fetcher.fetch_recent_commits(owner, repo).await
    }
}

/// Service for archiving repositories
pub struct RepoArchiver<S: RepoStorage> {
    storage: S,
}

impl<S: RepoStorage> RepoArchiver<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    pub async fn save_repo(&mut self, repo: &Repo) -> Result<(), AppError> {
        self.storage.store_repository(repo).await
    }
}

//! Service interfaces.

use crate::error::AppError;
use crate::model::{Commit, Issue, Repo};

#[allow(async_fn_in_trait)]
pub trait RepoFetcher {
    async fn fetch_top_repositories(
        &self,
        language: &str,
        per_page: u8,
    ) -> Result<Vec<Repo>, AppError>;
    async fn fetch_repo_forks(&self, owner: &str, repo: &str) -> Result<Vec<Repo>, AppError>;
}

#[allow(async_fn_in_trait)]
pub trait CommitFetcher {
    async fn fetch_recent_commits(&self, owner: &str, repo: &str) -> Result<Vec<Commit>, AppError>;
    async fn fetch_commit_with_files(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
    ) -> Result<Commit, AppError>;
}

#[allow(async_fn_in_trait)]
pub trait IssueFetcher {
    async fn fetch_open_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>, AppError>;
}

#[allow(async_fn_in_trait)]
pub trait RepoStorage {
    async fn store_repository(&mut self, repo: &Repo) -> Result<(), AppError>;
}

pub trait GitService: RepoFetcher + CommitFetcher + IssueFetcher {}

pub trait StorageService: RepoStorage {}


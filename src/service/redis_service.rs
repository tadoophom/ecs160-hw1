//! Redis storage service for persisting repository data.
//! Handles connection management and data storage operations for repositories, owners, and issues.
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::config::RedisConfig;
use crate::error::AppError;
use crate::model::{Issue, Owner, Repo};
use crate::service::traits::DataStorageService;

/// Manages Redis connections and storage operations
#[derive(Clone)]
pub struct RedisService {
    client: ConnectionManager,
}

impl RedisService {
    /// Creates a new Redis service with connection pool
    pub async fn new(config: RedisConfig) -> Result<Self, AppError> {
        let redis_client = redis::Client::open(config.url.as_str())
            .map_err(|e| AppError::Redis(format!("Failed to create Redis client: {e}")))?;

        let client = ConnectionManager::new(redis_client)
            .await
            .map_err(|e| AppError::Redis(format!("Failed to create connection manager: {e}")))?;

        Ok(Self { client })
    }

    /// Stores a single repository in Redis
    /// Uses format: repo:{owner}/{name} for repository data,
    /// author:{login} for owner data, and issue:{repo_id}:{issue_index} for issues
    pub async fn store_repository(&mut self, repo: &Repo) -> Result<(), AppError> {
        let repo_key = format!("repo:{}:{}", repo.owner.login, repo.name);

        // Store repository metadata
        self.client
            .hset_multiple::<_, _, _, ()>(
                &repo_key,
                &[
                    ("url", repo.html_url.as_str()),
                    ("name", repo.name.as_str()),
                    ("owner", repo.owner.login.as_str()),
                    ("language", &repo.language.as_deref().unwrap_or("unknown")),
                    ("stars", &repo.stargazers_count.to_string()),
                    ("forks", &repo.forks_count.to_string()),
                    ("open_issues", &repo.open_issues_count.to_string()),
                    ("full_name", repo.full_name.as_str()),
                ],
            )
            .await
            .map_err(|e| AppError::Redis(format!("Failed to store repo: {e}")))?;

        // Store owner/author data
        self.store_owner(&repo.owner).await?;

        // Store all issues for this repository
        for (idx, issue) in repo.issues.iter().enumerate() {
            self.store_issue(repo.id, idx, issue).await?;
        }

        Ok(())
    }

    /// Stores owner/author information in Redis
    async fn store_owner(&mut self, owner: &Owner) -> Result<(), AppError> {
        let key = format!("author:{}", owner.login);

        self.client
            .hset_multiple::<_, _, _, ()>(
                &key,
                &[
                    ("login", owner.login.as_str()),
                    ("id", &owner.id.to_string()),
                    ("url", owner.html_url.as_str()),
                    ("site_admin", &owner.site_admin.to_string()),
                ],
            )
            .await
            .map_err(|e| AppError::Redis(format!("Failed to store author: {e}")))?;

        Ok(())
    }

    /// Stores a single issue in Redis
    async fn store_issue(&mut self, repo_id: i64, index: usize, issue: &Issue) -> Result<(), AppError> {
        let key = format!("issue:{}:{}", repo_id, index);

        self.client
            .hset_multiple::<_, _, _, ()>(
                &key,
                &[
                    ("title", issue.title.as_str()),
                    ("body", &issue.body.as_deref().unwrap_or("")),
                    ("state", issue.state.as_str()),
                    ("url", &issue.html_url.as_deref().unwrap_or("")),
                    ("created_at", issue.created_at.as_str()),
                    ("updated_at", issue.updated_at.as_str()),
                ],
            )
            .await
            .map_err(|e| AppError::Redis(format!("Failed to store issue: {e}")))?;

        Ok(())
    }
}

impl DataStorageService for RedisService {
    async fn store_repository(&mut self, repo: &Repo) -> Result<(), AppError> {
        self.store_repository(repo).await
    }
}

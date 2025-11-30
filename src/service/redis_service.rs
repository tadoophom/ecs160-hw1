//! Redis storage.
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::config::RedisConfig;
use crate::error::AppError;
use crate::model::{Issue, Owner, Repo};
use crate::service::traits::DataStorageService;

#[derive(Clone)]
pub struct RedisService {
    client: ConnectionManager,
}

impl RedisService {
    pub async fn new(config: RedisConfig) -> Result<Self, AppError> {
        let redis_client = redis::Client::open(config.url.as_str())
            .map_err(|e| AppError::Redis(format!("Failed to create Redis client: {e}")))?;

        let client = ConnectionManager::new(redis_client)
            .await
            .map_err(|e| AppError::Redis(format!("Failed to create connection manager: {e}")))?;

        Ok(Self { client })
    }

    pub async fn store_repository(&mut self, repo: &Repo) -> Result<(), AppError> {
        let repo_key = format!("repo:{}:{}", repo.owner.login, repo.name);

        // Create comma-separated list of issue IDs
        let issues_list = repo
            .issues
            .iter()
            .map(|i| format!("iss-{}", i.id))
            .collect::<Vec<_>>()
            .join(",");

        self.client
            .hset_multiple::<_, _, _, ()>(
                &repo_key,
                &[
                    ("url", repo.html_url.as_str()),
                    ("Url", repo.html_url.as_str()), // Capitalized as requested
                    ("name", repo.name.as_str()),
                    ("owner", repo.owner.login.as_str()),
                    ("language", &repo.language.as_deref().unwrap_or("unknown")),
                    ("stars", &repo.stargazers_count.to_string()),
                    ("forks", &repo.forks_count.to_string()),
                    ("open_issues", &repo.open_issues_count.to_string()),
                    ("full_name", repo.full_name.as_str()),
                    ("Issues", &issues_list), // Capitalized as requested
                ],
            )
            .await
            .map_err(|e| AppError::Redis(format!("Failed to store repo: {e}")))?;

        self.store_owner(&repo.owner).await?;

        for issue in &repo.issues {
            self.store_issue(issue).await?;
        }

        Ok(())
    }

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
    async fn store_issue(
        &mut self,
        issue: &Issue,
    ) -> Result<(), AppError> {
        let key = format!("iss-{}", issue.id);

        self.client
            .hset_multiple::<_, _, _, ()>(
                &key,
                &[
                    ("issueId", key.as_str()), // Added issueId
                    ("title", issue.title.as_str()),
                    ("body", issue.body.as_deref().unwrap_or("")),
                    ("Description", issue.body.as_deref().unwrap_or("")),
                    ("description", issue.body.as_deref().unwrap_or("")), // Added description (lowercase)
                    ("state", issue.state.as_str()),
                    ("url", issue.html_url.as_deref().unwrap_or("")),
                    ("created_at", issue.created_at.as_str()),
                    ("Date", issue.created_at.as_str()),
                    ("updated_at", issue.updated_at.as_str()),
                    ("bug_type", "BUG"), // Added bug_type
                    ("filename", "unknown"), // Added filename
                    ("line", "0"), // Added line
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

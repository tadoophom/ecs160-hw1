//! Configuration loading.
use std::env;

use crate::error::AppError;

pub trait ConfigSource {
    fn get(&self, key: &str) -> Option<String>;
}

#[derive(Debug, Default)]
pub struct EnvSource;

impl EnvSource {
    pub fn with_dotenv() -> Self {
        let _ = dotenvy::dotenv();
        Self
    }
}

impl ConfigSource for EnvSource {
    fn get(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub github: GitHubConfig,
    pub redis: RedisConfig,
    pub clone: CloneConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, AppError> {
        let source = EnvSource::with_dotenv();
        Self::from_source(&source)
    }

    /// Allows callers (e.g. tests) to inject a custom configuration source.
    pub fn from_source(source: &impl ConfigSource) -> Result<Self, AppError> {
        Ok(Self {
            github: GitHubConfig::from_source(source)?,
            redis: RedisConfig::from_source(source)?,
            clone: CloneConfig::from_source(source)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GitHubConfig {
    pub token: Option<String>,
    pub api_base: String,
    pub user_agent: String,
}

impl GitHubConfig {
    const DEFAULT_API_BASE: &'static str = "https://api.github.com";
    const DEFAULT_USER_AGENT: &'static str = "ecs160-hw1-github-client/0.1";

    fn from_source(source: &impl ConfigSource) -> Result<Self, AppError> {
        let token = source.get("GITHUB_TOKEN");
        let api_base = source
            .get("GITHUB_API_BASE")
            .unwrap_or_else(|| Self::DEFAULT_API_BASE.to_string());
        let user_agent = source
            .get("GITHUB_USER_AGENT")
            .unwrap_or_else(|| Self::DEFAULT_USER_AGENT.to_string());

        Ok(Self {
            token,
            api_base,
            user_agent,
        })
    }

    /// Convenience helper for consumers that require an authenticated token.
    pub fn require_token(&self) -> Result<&str, AppError> {
        self.token.as_deref().ok_or(AppError::MissingGitHubToken)
    }
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
}

impl RedisConfig {
    const DEFAULT_REDIS_URL: &'static str = "redis://127.0.0.1:6379";

    fn from_source(source: &impl ConfigSource) -> Result<Self, AppError> {
        let url = source
            .get("REDIS_URL")
            .unwrap_or_else(|| Self::DEFAULT_REDIS_URL.to_string());

        Ok(Self { url })
    }
}

#[derive(Debug, Clone)]
pub struct CloneConfig {
    pub min_source_ratio: f64,
}

impl CloneConfig {
    const DEFAULT_MIN_SOURCE_RATIO: f64 = 0.05;

    fn from_source(source: &impl ConfigSource) -> Result<Self, AppError> {
        let min_source_ratio = source
            .get("CLONE_MIN_SOURCE_RATIO")
            .and_then(|s| s.parse().ok())
            .unwrap_or(Self::DEFAULT_MIN_SOURCE_RATIO);

        Ok(Self { min_source_ratio })
    }
}

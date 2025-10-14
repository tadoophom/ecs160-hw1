use reqwest::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};

use crate::config::GitHubConfig;
use crate::error::AppError;
use crate::models::{Commit, Issue, Repo};

/// Lightweight wrapper around `reqwest::Client` tailored for GitHub REST API access.
#[allow(dead_code)]
#[derive(Clone)]
pub struct GitHubClient {
    http: Client,
    config: GitHubConfig,
}

impl GitHubClient {
    /// Builds a new client instance using the provided configuration.
    pub fn new(config: GitHubConfig) -> Result<Self, AppError> {
        let http = Client::builder()
            .default_headers(Self::default_headers(&config)?)
            .build()
            .map_err(AppError::from)?;

        Ok(Self { http, config })
    }

    fn default_headers(config: &GitHubConfig) -> Result<HeaderMap, AppError> {
        let mut headers = HeaderMap::new();

        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&config.user_agent).map_err(|err| {
                AppError::Config(format!("invalid user agent header value: {err}"))
            })?,
        );

        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );

        if let Some(token) = &config.token {
            let value = format!("Bearer {token}");
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&value)
                    .map_err(|err| AppError::Config(format!("invalid token header: {err}")))?,
            );
        }

        Ok(headers)
    }

    /// Fetches the most popular repositories for a language via the GitHub Search API.
    pub async fn fetch_top_repositories(
        &self,
        _language: &str,
        _per_page: u8,
    ) -> Result<Vec<Repo>, AppError> {
        Err(AppError::NotImplemented)
    }

    /// Fetches forks for a repository stubbed for later implementation.
    pub async fn fetch_repo_forks(&self, _owner: &str, _repo: &str) -> Result<Vec<Repo>, AppError> {
        Err(AppError::NotImplemented)
    }

    /// Fetches recent commits for a repository stubbed for later implementation.
    pub async fn fetch_recent_commits(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<Vec<Commit>, AppError> {
        Err(AppError::NotImplemented)
    }

    /// Fetches open issues for a repository stubbed for later implementation.
    pub async fn fetch_open_issues(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<Vec<Issue>, AppError> {
        Err(AppError::NotImplemented)
    }
}

//! Implements the GitHub-facing service that handles HTTP calls and JSON parsing.
//! Offers high-level methods the app can call without dealing with networking details.
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, Url};
use serde_json::Value;

use crate::config::GitHubConfig;
use crate::error::AppError;
use crate::model::{Commit, Issue, Repo};
use crate::util::json::json_error;

/// Service wrapper around `reqwest::Client` tailored for GitHub REST API access.
#[allow(dead_code)]
#[derive(Clone)]
pub struct GitService {
    http: Client,
    config: GitHubConfig,
}

impl GitService {
    /// Builds a new service instance using the provided configuration.
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
        language: &str,
        per_page: u8,
    ) -> Result<Vec<Repo>, AppError> {
        let per_page = per_page.clamp(1, 100);

        let base_url = Url::parse(&self.config.api_base)
            .map_err(|err| AppError::Config(format!("invalid GitHub API base url: {err}")))?;

        let url = base_url.join("search/repositories").map_err(|err| {
            AppError::Config(format!("failed to construct search endpoint URL: {err}"))
        })?;

        let response = self
            .http
            .get(url)
            .query(&[
                ("q", format!("language:{language}")),
                ("sort", "stars".to_string()),
                ("order", "desc".to_string()),
                ("per_page", per_page.to_string()),
                ("page", "1".to_string()),
            ])
            .send()
            .await
            .map_err(AppError::from)?;

        let response = response.error_for_status().map_err(AppError::from)?;
        let body = response.text().await.map_err(AppError::from)?;
        let root: Value = serde_json::from_str(&body).map_err(AppError::from)?;

        let items = root
            .get("items")
            .and_then(Value::as_array)
            .ok_or_else(|| json_error("GitHub search response missing `items` array"))?;

        items
            .iter()
            .map(Repo::from_json)
            .collect::<Result<Vec<_>, _>>()
    }

    /// Fetches forks for a repository.
    pub async fn fetch_repo_forks(&self, owner: &str, repo: &str) -> Result<Vec<Repo>, AppError> {
        let base_url = Url::parse(&self.config.api_base)
            .map_err(|err| AppError::Config(format!("invalid GitHub API base url: {err}")))?;

        let url = base_url
            .join(&format!("repos/{owner}/{repo}/forks"))
            .map_err(|err| {
                AppError::Config(format!("failed to construct forks endpoint URL: {err}"))
            })?;

        let response = self
            .http
            .get(url)
            .query(&[
                ("per_page", "100".to_string()),
                ("page", "1".to_string()),
                ("sort", "newest".to_string()),
            ])
            .send()
            .await
            .map_err(AppError::from)?;

        let response = response.error_for_status().map_err(AppError::from)?;
        let body = response.text().await.map_err(AppError::from)?;
        let root: Value = serde_json::from_str(&body).map_err(AppError::from)?;

        let items = root
            .as_array()
            .ok_or_else(|| json_error("GitHub forks response was not an array"))?;

        items
            .iter()
            .map(Repo::from_json)
            .collect::<Result<Vec<_>, _>>()
    }

    /// Fetches recent commits for a repository.
    pub async fn fetch_recent_commits(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<Commit>, AppError> {
        let base_url = Url::parse(&self.config.api_base)
            .map_err(|err| AppError::Config(format!("invalid GitHub API base url: {err}")))?;

        let url = base_url
            .join(&format!("repos/{owner}/{repo}/commits"))
            .map_err(|err| {
                AppError::Config(format!("failed to construct commits endpoint URL: {err}"))
            })?;

        let response = self
            .http
            .get(url)
            .query(&[("per_page", "50".to_string()), ("page", "1".to_string())])
            .send()
            .await
            .map_err(AppError::from)?;

        let response = response.error_for_status().map_err(AppError::from)?;
        let body = response.text().await.map_err(AppError::from)?;
        let root: Value = serde_json::from_str(&body).map_err(AppError::from)?;

        let items = root
            .as_array()
            .ok_or_else(|| json_error("GitHub commits response was not an array"))?;

        items
            .iter()
            .map(Commit::from_json)
            .collect::<Result<Vec<_>, _>>()
    }

    /// Fetches open issues for a repository.
    pub async fn fetch_open_issues(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<Issue>, AppError> {
        let base_url = Url::parse(&self.config.api_base)
            .map_err(|err| AppError::Config(format!("invalid GitHub API base url: {err}")))?;

        let url = base_url
            .join(&format!("repos/{owner}/{repo}/issues"))
            .map_err(|err| {
                AppError::Config(format!("failed to construct issues endpoint URL: {err}"))
            })?;

        let response = self
            .http
            .get(url)
            .query(&[
                ("state", "open".to_string()),
                ("per_page", "100".to_string()),
                ("page", "1".to_string()),
            ])
            .send()
            .await
            .map_err(AppError::from)?;

        let response = response.error_for_status().map_err(AppError::from)?;
        let body = response.text().await.map_err(AppError::from)?;
        let root: Value = serde_json::from_str(&body).map_err(AppError::from)?;

        let items = root
            .as_array()
            .ok_or_else(|| json_error("GitHub issues response was not an array"))?;

        items
            .iter()
            .map(Issue::from_json)
            .collect::<Result<Vec<_>, _>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;

    fn service_with_base(base_url: &str) -> GitService {
        let config = GitHubConfig {
            token: None,
            api_base: base_url.to_string(),
            user_agent: "ecs160-test-agent/0.1".to_string(),
        };

        GitService::new(config).expect("failed to construct test client")
    }

    fn sample_response() -> serde_json::Value {
        json!({
            "total_count": 1,
            "incomplete_results": false,
            "items": [
                {
                    "id": 42,
                    "name": "repo-one",
                    "full_name": "octocat/repo-one",
                    "html_url": "https://example.com/repo-one",
                    "forks_count": 5,
                    "stargazers_count": 100,
                    "open_issues_count": 7,
                    "language": "Rust",
                    "owner": {
                        "login": "octocat",
                        "id": 1,
                        "html_url": "https://github.com/octocat",
                        "site_admin": false
                    }
                }
            ]
        })
    }

    #[tokio::test]
    async fn fetch_top_repositories_returns_items() {
        let server = MockServer::start_async().await;

        let mock = server
            .mock_async(|when, then| {
                when.method(GET)
                    .path("/search/repositories")
                    .query_param("q", "language:Rust")
                    .query_param("sort", "stars")
                    .query_param("order", "desc")
                    .query_param("per_page", "10")
                    .query_param("page", "1");

                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(sample_response());
            })
            .await;

        let service = service_with_base(&server.base_url());
        let repos = service.fetch_top_repositories("Rust", 10).await.unwrap();

        assert_eq!(repos.len(), 1);
        let repo = &repos[0];
        assert_eq!(repo.name, "repo-one");
        assert_eq!(repo.owner.login, "octocat");
        assert_eq!(repo.slug(), "octocat/repo-one");

        mock.assert();
    }

    #[tokio::test]
    async fn fetch_top_repositories_clamps_per_page_to_max() {
        let server = MockServer::start_async().await;

        let mock = server
            .mock_async(|when, then| {
                when.method(GET)
                    .path("/search/repositories")
                    .query_param("per_page", "100");

                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(sample_response());
            })
            .await;

        let service = service_with_base(&server.base_url());
        let repos = service
            .fetch_top_repositories("Rust", 200)
            .await
            .expect("request should succeed");

        assert_eq!(repos.len(), 1);
        mock.assert();
    }
}

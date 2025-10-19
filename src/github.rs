use reqwest::Client;
use reqwest::Url;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;

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
        let language = _language;
        let per_page = _per_page.clamp(1, 100);

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
        let parsed: SearchRepositoriesResponse =
            serde_json::from_str(&body).map_err(AppError::from)?;

        Ok(parsed.items)
    }

    /// Fetches forks for a repository stubbed for later implementation.
    pub async fn fetch_repo_forks(&self, _owner: &str, _repo: &str) -> Result<Vec<Repo>, AppError> {
        let owner = _owner;
        let repo = _repo;

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
        let parsed: Vec<Repo> = serde_json::from_str(&body).map_err(AppError::from)?;

        Ok(parsed)
    }

    /// Fetches recent commits for a repository stubbed for later implementation.
    pub async fn fetch_recent_commits(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<Vec<Commit>, AppError> {
        let owner = _owner;
        let repo = _repo;

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
        let parsed: Vec<Commit> = serde_json::from_str(&body).map_err(AppError::from)?;

        Ok(parsed)
    }

    /// Fetches open issues for a repository stubbed for later implementation.
    pub async fn fetch_open_issues(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<Vec<Issue>, AppError> {
        let owner = _owner;
        let repo = _repo;

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
        let parsed: Vec<Issue> = serde_json::from_str(&body).map_err(AppError::from)?;

        Ok(parsed)
    }
}

#[derive(Debug, Deserialize)]
struct SearchRepositoriesResponse {
    #[allow(dead_code)]
    total_count: u64,
    #[allow(dead_code)]
    incomplete_results: bool,
    items: Vec<Repo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;

    fn client_with_base(base_url: &str) -> GitHubClient {
        let config = GitHubConfig {
            token: None,
            api_base: base_url.to_string(),
            user_agent: "ecs160-test-agent/0.1".to_string(),
        };

        GitHubClient::new(config).expect("failed to construct test client")
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

        let client = client_with_base(&server.base_url());
        let repos = client.fetch_top_repositories("Rust", 10).await.unwrap();

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

        let client = client_with_base(&server.base_url());
        let repos = client
            .fetch_top_repositories("Rust", 200)
            .await
            .expect("request should succeed");

        assert_eq!(repos.len(), 1);
        mock.assert();
    }
}

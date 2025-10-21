//! Integration-style tests that exercise the application workflow via mocks.
//! They ensure the app layer composes the service and models correctly.
use ecs160_hw1::app::{collect_language_snapshot, ForkSummary, LanguageSnapshot};
use ecs160_hw1::config::GitHubConfig;
use ecs160_hw1::GitService;
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

fn sample_search_response() -> serde_json::Value {
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

fn sample_forks_response() -> serde_json::Value {
    json!([
        {
            "id": 1,
            "name": "repo-one",
            "full_name": "forker/repo-one",
            "html_url": "https://example.com/fork-one",
            "forks_count": 0,
            "stargazers_count": 0,
            "open_issues_count": 0,
            "language": "Rust",
            "owner": {
                "login": "forker",
                "id": 2,
                "html_url": "https://github.com/forker",
                "site_admin": false
            }
        },
        {
            "id": 2,
            "name": "repo-one",
            "full_name": "someone/repo-one",
            "html_url": "https://example.com/fork-two",
            "forks_count": 0,
            "stargazers_count": 0,
            "open_issues_count": 0,
            "language": "Rust",
            "owner": {
                "login": "someone",
                "id": 3,
                "html_url": "https://github.com/someone",
                "site_admin": false
            }
        }
    ])
}

#[tokio::test]
async fn collect_language_snapshot_fetches_top_repo_and_forks() {
    let server = MockServer::start_async().await;

    let search_mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/search/repositories")
                .query_param("q", "language:Rust")
                .query_param("sort", "stars")
                .query_param("order", "desc")
                .query_param("per_page", "5")
                .query_param("page", "1");

            then.status(200)
                .header("content-type", "application/json")
                .json_body(sample_search_response());
        })
        .await;

    let forks_mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/repos/octocat/repo-one/forks")
                .query_param("per_page", "100")
                .query_param("page", "1")
                .query_param("sort", "newest");

            then.status(200)
                .header("content-type", "application/json")
                .json_body(sample_forks_response());
        })
        .await;

    let service = service_with_base(&server.base_url());
    let snapshot = collect_language_snapshot(&service, "Rust", 5)
        .await
        .expect("snapshot should be collected");

    assert_eq!(
        snapshot,
        LanguageSnapshot {
            language: "Rust".to_string(),
            repo_count: 1,
            top_repo_slug: Some("octocat/repo-one".to_string()),
            forks: Some(Ok(ForkSummary {
                total: 2,
                sample: vec![
                    "forker/repo-one".to_string(),
                    "someone/repo-one".to_string()
                ],
            })),
        }
    );

    search_mock.assert();
    forks_mock.assert();
}

#[tokio::test]
async fn collect_language_snapshot_handles_fork_errors() {
    let server = MockServer::start_async().await;

    let search_mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/search/repositories");

            then.status(200)
                .header("content-type", "application/json")
                .json_body(sample_search_response());
        })
        .await;

    let forks_mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/repos/octocat/repo-one/forks");

            then.status(500);
        })
        .await;

    let service = service_with_base(&server.base_url());
    let snapshot = collect_language_snapshot(&service, "Rust", 5)
        .await
        .expect("snapshot should be collected even if fork fetch fails");

    assert_eq!(snapshot.top_repo_slug.as_deref(), Some("octocat/repo-one"));
    assert!(snapshot.forks.is_some());
    let error = snapshot.forks.unwrap().unwrap_err();
    assert!(error.contains("status"));

    search_mock.assert();
    forks_mock.assert();
}

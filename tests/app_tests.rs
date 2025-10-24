//! App tests.
use ecs160_hw1::app::collect_language_report;
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

fn sample_commits_response() -> serde_json::Value {
    json!([
        {
            "sha": "abc123",
            "url": "https://api.github.com/repos/octocat/repo-one/commits/abc123",
            "html_url": "https://github.com/octocat/repo-one/commit/abc123",
            "commit": {
                "message": "Initial commit",
                "author": {
                    "name": "Coder",
                    "email": "coder@example.com",
                    "date": "2024-01-01T00:00:00Z"
                },
                "committer": {
                    "name": "Coder",
                    "email": "coder@example.com",
                    "date": "2024-01-01T00:00:00Z"
                }
            }
        }
    ])
}

fn sample_commit_detail_response() -> serde_json::Value {
    json!({
        "sha": "abc123",
        "url": "https://api.github.com/repos/octocat/repo-one/commits/abc123",
        "html_url": "https://github.com/octocat/repo-one/commit/abc123",
        "commit": {
            "message": "Initial commit",
            "author": {
                "name": "Coder",
                "email": "coder@example.com",
                "date": "2024-01-01T00:00:00Z"
            },
            "committer": {
                "name": "Coder",
                "email": "coder@example.com",
                "date": "2024-01-01T00:00:00Z"
            }
        },
        "files": [
            {
                "filename": "src/main.rs",
                "additions": 10,
                "deletions": 0,
                "changes": 10,
                "status": "added"
            }
        ]
    })
}

fn sample_issues_response() -> serde_json::Value {
    json!([
        {
            "title": "Bug report",
            "body": "Something broke",
            "state": "open",
            "html_url": "https://github.com/octocat/repo-one/issues/1",
            "created_at": "2024-01-02T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z"
        }
    ])
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

fn empty_commits_response() -> serde_json::Value {
    json!([])
}

#[tokio::test]
async fn collect_language_report_fetches_repo_details() {
    let server = MockServer::start_async().await;

    let search_mock = server
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
                .json_body(sample_search_response());
        })
        .await;

    let commits_mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/repos/octocat/repo-one/commits")
                .query_param("per_page", "50")
                .query_param("page", "1");

            then.status(200)
                .header("content-type", "application/json")
                .json_body(sample_commits_response());
        })
        .await;

    let commit_detail_mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/repos/octocat/repo-one/commits/abc123");

            then.status(200)
                .header("content-type", "application/json")
                .json_body(sample_commit_detail_response());
        })
        .await;

    let issues_mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/repos/octocat/repo-one/issues")
                .query_param("state", "open")
                .query_param("per_page", "100")
                .query_param("page", "1");

            then.status(200)
                .header("content-type", "application/json")
                .json_body(sample_issues_response());
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

    let fork_commits_mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/repos/forker/repo-one/commits")
                .query_param("per_page", "50")
                .query_param("page", "1");

            then.status(200)
                .header("content-type", "application/json")
                .json_body(empty_commits_response());
        })
        .await;

    let fork_commits_mock_two = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/repos/someone/repo-one/commits")
                .query_param("per_page", "50")
                .query_param("page", "1");

            then.status(200)
                .header("content-type", "application/json")
                .json_body(empty_commits_response());
        })
        .await;

    let service = service_with_base(&server.base_url());
    let report = collect_language_report(&service, "Rust")
        .await
        .expect("report should be collected");

    assert_eq!(report.language, "Rust");
    assert_eq!(report.repos.len(), 1);
    assert_eq!(report.total_stars, 100);
    assert_eq!(report.total_forks, 5);
    assert_eq!(report.total_open_issues, 1);
    assert_eq!(report.total_repo_commits, 1);
    assert_eq!(report.new_fork_commits, 0);
    assert_eq!(report.repo_metrics.len(), 1);
    let repo_metrics = &report.repo_metrics[0];
    assert_eq!(repo_metrics.slug, "octocat/repo-one");
    assert_eq!(repo_metrics.top_files, vec!["src/main.rs".to_string()]);

    let repo = &report.repos[0];
    assert_eq!(repo.slug(), "octocat/repo-one");
    assert_eq!(repo.commit_count, 1);
    assert_eq!(repo.recent_commits.len(), 1);
    assert_eq!(repo.issues.len(), 1);
    assert_eq!(repo.forks.len(), 2);

    search_mock.assert();
    commits_mock.assert();
    commit_detail_mock.assert();
    issues_mock.assert();
    forks_mock.assert();
    fork_commits_mock.assert();
    fork_commits_mock_two.assert();
}

#[tokio::test]
async fn collect_language_report_handles_fork_errors() {
    let server = MockServer::start_async().await;

    let search_mock = server
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
                .json_body(sample_search_response());
        })
        .await;

    let commits_mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/repos/octocat/repo-one/commits")
                .query_param("per_page", "50")
                .query_param("page", "1");

            then.status(200)
                .header("content-type", "application/json")
                .json_body(sample_commits_response());
        })
        .await;

    let commit_detail_mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/repos/octocat/repo-one/commits/abc123");

            then.status(200)
                .header("content-type", "application/json")
                .json_body(sample_commit_detail_response());
        })
        .await;

    let issues_mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/repos/octocat/repo-one/issues")
                .query_param("state", "open")
                .query_param("per_page", "100")
                .query_param("page", "1");

            then.status(200)
                .header("content-type", "application/json")
                .json_body(sample_issues_response());
        })
        .await;

    let forks_mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/repos/octocat/repo-one/forks")
                .query_param("per_page", "100")
                .query_param("page", "1")
                .query_param("sort", "newest");

            then.status(500);
        })
        .await;

    let service = service_with_base(&server.base_url());
    let report = collect_language_report(&service, "Rust")
        .await
        .expect("report should still be collected when forks fail");

    assert_eq!(report.repos.len(), 1);
    assert!(report.repos[0].forks.is_empty());
    assert_eq!(report.new_fork_commits, 0);
    assert_eq!(report.repo_metrics.len(), 1);
    assert!(report.repo_metrics[0]
        .top_files
        .contains(&"src/main.rs".to_string()));

    search_mock.assert();
    commits_mock.assert();
    commit_detail_mock.assert();
    issues_mock.assert();
    forks_mock.assert();
}

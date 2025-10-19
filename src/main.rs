use ecs160_hw1::config::AppConfig;
use ecs160_hw1::error::AppError;
use ecs160_hw1::github::GitHubClient;

const TARGET_LANGUAGES: &[&str] = &["Java", "C", "C++", "Rust"];
const TOP_N: u8 = 10;
const FORK_DISPLAY_LIMIT: usize = 5;

#[derive(Debug, PartialEq)]
struct LanguageSnapshot {
    language: String,
    repo_count: usize,
    top_repo_slug: Option<String>,
    forks: Option<Result<ForkSummary, String>>,
}

#[derive(Debug, PartialEq)]
struct ForkSummary {
    total: usize,
    sample: Vec<String>,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("application error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    let config = AppConfig::load()?;
    let client = GitHubClient::new(config.github.clone())?;

    for &language in TARGET_LANGUAGES {
        match collect_language_snapshot(&client, language, TOP_N).await {
            Ok(snapshot) => print_snapshot(&snapshot),
            Err(AppError::NotImplemented) => {
                println!("`fetch_top_repositories` is not implemented yet; skipping `{language}`.");
            }
            Err(err) => {
                eprintln!("failed to process `{language}`: {err}");
            }
        }
    }

    Ok(())
}

async fn collect_language_snapshot(
    client: &GitHubClient,
    language: &str,
    per_page: u8,
) -> Result<LanguageSnapshot, AppError> {
    let repos = client.fetch_top_repositories(language, per_page).await?;
    let top_repo = repos.first();

    let forks = if let Some(repo) = top_repo {
        Some(
            client
                .fetch_repo_forks(&repo.owner.login, &repo.name)
                .await
                .map(|forks| ForkSummary {
                    total: forks.len(),
                    sample: forks
                        .iter()
                        .take(FORK_DISPLAY_LIMIT)
                        .map(|fork| fork.slug())
                        .collect(),
                })
                .map_err(|err| err.to_string()),
        )
    } else {
        None
    };

    Ok(LanguageSnapshot {
        language: language.to_string(),
        repo_count: repos.len(),
        top_repo_slug: top_repo.map(|repo| repo.slug()),
        forks,
    })
}

fn print_snapshot(snapshot: &LanguageSnapshot) {
    println!(
        "language `{}`: retrieved {} repositories",
        snapshot.language, snapshot.repo_count
    );

    match (snapshot.top_repo_slug.as_deref(), &snapshot.forks) {
        (Some(slug), Some(Ok(summary))) => {
            println!("top repo `{slug}`:");
            println!("  forks retrieved: {}", summary.total);
            if !summary.sample.is_empty() {
                println!("  sample forks:");
                for fork in &summary.sample {
                    println!("    - {fork}");
                }
            } else {
                println!("  no forks returned");
            }
        }
        (Some(slug), Some(Err(message))) => {
            println!("top repo `{slug}`:");
            println!("  failed to fetch forks: {message}");
        }
        (Some(_), None) => {
            println!("top repo data unavailable");
        }
        (None, _) => println!("no repositories returned by GitHub search"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs160_hw1::config::GitHubConfig;
    use ecs160_hw1::github::GitHubClient;
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

        let client = client_with_base(&server.base_url());
        let snapshot = collect_language_snapshot(&client, "Rust", 5)
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

        let client = client_with_base(&server.base_url());
        let snapshot = collect_language_snapshot(&client, "Rust", 5)
            .await
            .expect("snapshot should be collected even if fork fetch fails");

        assert_eq!(snapshot.top_repo_slug.as_deref(), Some("octocat/repo-one"));
        assert!(snapshot.forks.is_some());
        let error = snapshot.forks.unwrap().unwrap_err();
        assert!(error.contains("status"));

        search_mock.assert();
        forks_mock.assert();
    }
}

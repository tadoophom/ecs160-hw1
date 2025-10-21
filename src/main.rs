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
mod tests;

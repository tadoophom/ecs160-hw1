//! Coordinates the high-level workflow: load config, call services,
//! and format language snapshots for console output.
use crate::config::AppConfig;
use crate::error::AppError;
use crate::service::GitService;

const TARGET_LANGUAGES: &[&str] = &["Java", "C", "C++", "Rust"];
const TOP_N: u8 = 10;
const FORK_DISPLAY_LIMIT: usize = 5;

#[derive(Debug, PartialEq)]
pub struct LanguageSnapshot {
    pub language: String,
    pub repo_count: usize,
    pub top_repo_slug: Option<String>,
    pub forks: Option<Result<ForkSummary, String>>,
}

#[derive(Debug, PartialEq)]
pub struct ForkSummary {
    pub total: usize,
    pub sample: Vec<String>,
}

pub async fn run() -> Result<(), AppError> {
    let config = AppConfig::load()?;
    let service = GitService::new(config.github.clone())?;

    for &language in TARGET_LANGUAGES {
        match collect_language_snapshot(&service, language, TOP_N).await {
            Ok(snapshot) => print_snapshot(&snapshot),
            Err(AppError::NotImplemented) => {
                println!(
                    "`fetch_top_repositories` is not implemented yet; skipping `{language}`."
                );
            }
            Err(err) => {
                eprintln!("failed to process `{language}`: {err}");
            }
        }
    }

    Ok(())
}

pub async fn collect_language_snapshot(
    service: &GitService,
    language: &str,
    per_page: u8,
) -> Result<LanguageSnapshot, AppError> {
    let repos = service.fetch_top_repositories(language, per_page).await?;
    let top_repo = repos.first();

    let forks = if let Some(repo) = top_repo {
        Some(
            service
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

pub fn print_snapshot(snapshot: &LanguageSnapshot) {
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

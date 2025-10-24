//! Main application.

use crate::config::AppConfig;
use crate::error::AppError;
use crate::model::Repo;
use crate::service::{GitService, RedisService};

pub mod clone;
pub mod output;
pub mod repo_fetcher;
pub mod stats;

use output::OutputFormatter;
use repo_fetcher::RepoFetcher;
use stats::StatsCalculator;

const TARGET_LANGUAGES: &[&str] = &["Java", "C", "C++", "Rust"];

#[derive(Debug)]
pub struct LanguageReport {
    pub language: String,
    pub repos: Vec<Repo>,
    pub total_stars: u64,
    pub total_forks: u64,
    pub total_open_issues: usize,
    pub total_repo_commits: usize,
    pub new_fork_commits: usize,
    pub repo_metrics: Vec<RepoMetrics>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoMetrics {
    pub slug: String,
    pub top_files: Vec<String>,
}

pub async fn run() -> Result<(), AppError> {
    let config = AppConfig::load()?;
    let service = GitService::new(config.github.clone())?;
    let mut redis = RedisService::new(config.redis.clone()).await?;

    println!("=== Part A: Fetching GitHub Repository Data ===\n");

    let mut language_reports = Vec::new();

    for &language in TARGET_LANGUAGES {
        println!("Processing language: {}", language);
        println!("{}", "=".repeat(50));

        match collect_language_report(&service, language).await {
            Ok(report) => {
                println!(
                    "✓ Successfully fetched {} repositories for {}",
                    report.repos.len(),
                    language
                );
                OutputFormatter::print_summary(&report);
                language_reports.push(report);
            }
            Err(err) => {
                eprintln!("✗ Failed to process {}: {}", language, err);
            }
        }

        println!();
    }

    // Part C: Clone and inspect repositories
    let clone_base_dir = std::path::Path::new("./cloned_repos");
    let cloned_repos = clone::clone_best_repos(
        &language_reports,
        clone_base_dir,
        config.clone.min_source_ratio,
    )
    .await?;

    // Part D: Store results in Redis (only store the cloned repos, not all 10)
    println!("\n=== Part D: Storing Results in Redis ===\n");
    store_cloned_repos_in_redis(&mut redis, &cloned_repos).await?;

    Ok(())
}

pub async fn collect_language_report(
    service: &GitService,
    language: &str,
) -> Result<LanguageReport, AppError> {
    let fetcher = RepoFetcher::new(service);
    let repos = fetcher.fetch_language_data(language).await?;

    Ok(StatsCalculator::build_language_report(language, repos))
}

async fn store_cloned_repos_in_redis(
    redis: &mut RedisService,
    cloned_repos: &[Repo],
) -> Result<(), AppError> {
    if cloned_repos.is_empty() {
        println!("⚠ No repositories were cloned, skipping Redis storage");
        return Ok(());
    }

    println!(
        "  Storing {} most popular source code repositories...",
        cloned_repos.len()
    );

    for repo in cloned_repos {
        redis.store_repository(repo).await?;
        println!(
            "    ✓ Stored {}/{} ({} stars)",
            repo.owner.login, repo.name, repo.stargazers_count
        );
    }

    println!(
        "\n✓ Successfully stored {} repositories in Redis",
        cloned_repos.len()
    );
    Ok(())
}

//! Coordinates the high-level workflow: load config, call services,
//! fetch all required data for Part A
use crate::config::AppConfig;
use crate::error::AppError;
use std::collections::HashMap;

use crate::model::Repo;
use crate::service::GitService;

const TARGET_LANGUAGES: &[&str] = &["Java", "C", "C++", "Rust"];
const TOP_N: u8 = 10;
const MAX_FORKS_TO_PROCESS: usize = 20;
const MAX_COMMITS_WITH_FILES: usize = 50;

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

    println!("=== Part A: Fetching GitHub Repository Data ===\n");

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
                print_summary(&report);
            }
            Err(err) => {
                eprintln!("✗ Failed to process {}: {}", language, err);
            }
        }

        println!();
    }
    Ok(())
}

pub async fn collect_language_report(
    service: &GitService,
    language: &str,
) -> Result<LanguageReport, AppError> {
    println!("  [1/4] Fetching top {} repositories...", TOP_N);
    let mut repos = service.fetch_top_repositories(language, TOP_N).await?;
    println!("      ✓ Found {} repositories", repos.len());

    println!("  [2/4] Fetching commits and issues for each repository...");
    for repo in &mut repos {
        match service
            .fetch_recent_commits(&repo.owner.login, &repo.name)
            .await
        {
            Ok(commits) => {
                println!("      ✓ {}: {} commits", repo.slug(), commits.len());

                repo.commit_count = commits.len() as u64;
                let mut detailed_commits = Vec::new();
                for commit in commits.iter().take(MAX_COMMITS_WITH_FILES) {
                    match service
                        .fetch_commit_with_files(&repo.owner.login, &repo.name, &commit.sha)
                        .await
                    {
                        Ok(detailed) => detailed_commits.push(detailed),
                        Err(e) => {
                            eprintln!(
                                "        ⚠ Failed to fetch details for commit {}: {}",
                                &commit.sha[..7],
                                e
                            );
                        }
                    }
                }
                repo.recent_commits = detailed_commits;
            }
            Err(e) => {
                eprintln!("      ✗ Failed to fetch commits for {}: {}", repo.slug(), e);
            }
        }

        match service
            .fetch_open_issues(&repo.owner.login, &repo.name)
            .await
        {
            Ok(issues) => {
                repo.issues = issues;
                println!("      ✓ {}: {} open issues", repo.slug(), repo.issues.len());
            }
            Err(e) => {
                eprintln!("      ✗ Failed to fetch issues for {}: {}", repo.slug(), e);
            }
        }
    }

    println!("  [3/4] Fetching forks for each repository...");
    for repo in &mut repos {
        match service
            .fetch_repo_forks(&repo.owner.login, &repo.name)
            .await
        {
            Ok(forks) => {
                println!("      ✓ {}: {} forks", repo.slug(), forks.len());
                repo.forks = forks;
            }
            Err(e) => {
                eprintln!("      ✗ Failed to fetch forks for {}: {}", repo.slug(), e);
            }
        }
    }

    println!("  [4/4] Fetching commits for forked repositories...");
    for repo in &mut repos {
        for fork in repo.forks.iter_mut().take(MAX_FORKS_TO_PROCESS) {
            match service
                .fetch_recent_commits(&fork.owner.login, &fork.name)
                .await
            {
                Ok(commits) => {
                    fork.commit_count = commits.len() as u64;
                    fork.recent_commits = commits;
                }
                Err(e) => {
                    eprintln!(
                        "      ⚠ Failed to fetch commits for fork {}: {}",
                        fork.slug(),
                        e
                    );
                }
            }
        }
        let forks_with_commits = repo.forks.iter().filter(|f| f.commit_count > 0).count();
        if forks_with_commits > 0 {
            println!(
                "      ✓ {}: fetched commits for {}/{} forks",
                repo.slug(),
                forks_with_commits,
                repo.forks.len().min(MAX_FORKS_TO_PROCESS)
            );
        }
    }

    let total_stars: u64 = repos.iter().map(|r| r.stargazers_count).sum();
    let total_forks: u64 = repos.iter().map(|r| r.forks_count).sum();
    let total_open_issues: usize = repos.iter().map(|r| r.issues.len()).sum();
    let (repo_metrics, new_fork_commits) = summarize_repos(&repos);
    let total_repo_commits: usize = repos.iter().map(|r| r.commit_count as usize).sum();

    Ok(LanguageReport {
        language: language.to_string(),
        repos,
        total_stars,
        total_forks,
        total_open_issues,
        total_repo_commits,
        new_fork_commits,
        repo_metrics,
    })
}

fn print_summary(report: &LanguageReport) {
    println!("Language: {}", report.language);
    println!("Total stars: {}", report.total_stars);
    println!("Total forks: {}", report.total_forks);
    println!("Open issues in top-10 repos: {}", report.total_open_issues);
    println!("Total commits fetched: {}", report.total_repo_commits);
    println!("New commits in forked repos: {}", report.new_fork_commits);
    println!("Top-3 Most modified file per repo:");
    for metrics in &report.repo_metrics {
        println!("  Repo {}:", metrics.slug);
        if metrics.top_files.is_empty() {
            println!("    No files modified in recent commits");
        } else {
            for (idx, file) in metrics.top_files.iter().enumerate() {
                println!("    File {}: {}", idx + 1, file);
            }
        }
    }
}

fn summarize_repos(repos: &[Repo]) -> (Vec<RepoMetrics>, usize) {
    let mut metrics = Vec::with_capacity(repos.len());
    let mut fork_commit_total = 0usize;

    for repo in repos {
        let top_files = compute_top_files(repo);
        let fork_commits: usize = repo
            .forks
            .iter()
            .take(MAX_FORKS_TO_PROCESS)
            .map(|fork| fork.commit_count as usize)
            .sum();
        fork_commit_total += fork_commits;

        metrics.push(RepoMetrics {
            slug: repo.slug(),
            top_files,
        });
    }

    (metrics, fork_commit_total)
}

fn compute_top_files(repo: &Repo) -> Vec<String> {
    let mut by_file: HashMap<String, i64> = HashMap::new();

    for commit in &repo.recent_commits {
        for file in &commit.files {
            let mut score = file.changes;
            if score == 0 {
                score = file.additions + file.deletions;
            }
            by_file
                .entry(file.filename.clone())
                .and_modify(|total| *total += score)
                .or_insert(score);
        }
    }

    let mut items: Vec<(String, i64)> = by_file.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    items.into_iter().map(|(name, _)| name).take(3).collect()
}

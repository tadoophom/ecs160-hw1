//! Coordinates the high-level workflow: load config, call services,
//! fetch all required data for Part A
use crate::config::AppConfig;
use crate::error::AppError;
use crate::model::Repo;
use crate::service::GitService;

const TARGET_LANGUAGES: &[&str] = &["Java", "C", "C++", "Rust"];
const TOP_N: u8 = 10;
const MAX_FORKS_TO_PROCESS: usize = 20;
const MAX_COMMITS_WITH_FILES: usize = 50;

pub async fn run() -> Result<(), AppError> {
    let config = AppConfig::load()?;
    let service = GitService::new(config.github.clone())?;

    println!("=== Part A: Fetching GitHub Repository Data ===\n");

    for &language in TARGET_LANGUAGES {
        println!("Processing language: {}", language);
        println!("{}", "=".repeat(50));
        
        match fetch_language_data(&service, language).await {
            Ok(repos) => {
                println!("✓ Successfully fetched {} repositories for {}", repos.len(), language);
                print_summary(&repos, language);
            }
            Err(err) => {
                eprintln!("✗ Failed to process {}: {}", language, err);
            }
        }
        
        println!();
    }
    Ok(())
}

async fn fetch_language_data(
    service: &GitService,
    language: &str,
) -> Result<Vec<Repo>, AppError> {
    println!("  [1/4] Fetching top {} repositories...", TOP_N);
    let mut repos = service.fetch_top_repositories(language, TOP_N).await?;
    println!("      ✓ Found {} repositories", repos.len());

    println!("  [2/4] Fetching commits and issues for each repository...");
    for repo in &mut repos {
        match service.fetch_recent_commits(&repo.owner.login, &repo.name).await {
            Ok(commits) => {
                println!("      ✓ {}: {} commits", repo.slug(), commits.len());
                
                let mut detailed_commits = Vec::new();
                for commit in commits.iter().take(MAX_COMMITS_WITH_FILES) {
                    match service.fetch_commit_with_files(&repo.owner.login, &repo.name, &commit.sha).await {
                        Ok(detailed) => detailed_commits.push(detailed),
                        Err(e) => {
                            eprintln!("        ⚠ Failed to fetch details for commit {}: {}", &commit.sha[..7], e);
                        }
                    }
                }
                repo.recent_commits = detailed_commits;
                repo.commit_count = repo.recent_commits.len() as u64;
            }
            Err(e) => {
                eprintln!("      ✗ Failed to fetch commits for {}: {}", repo.slug(), e);
            }
        }

        match service.fetch_open_issues(&repo.owner.login, &repo.name).await {
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
        match service.fetch_repo_forks(&repo.owner.login, &repo.name).await {
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
            match service.fetch_recent_commits(&fork.owner.login, &fork.name).await {
                Ok(commits) => {
                    fork.recent_commits = commits;
                    fork.commit_count = fork.recent_commits.len() as u64;
                }
                Err(e) => {
                    eprintln!("      ⚠ Failed to fetch commits for fork {}: {}", fork.slug(), e);
                }
            }
        }
        let forks_with_commits = repo.forks.iter()
            .filter(|f| f.commit_count > 0)
            .count();
        if forks_with_commits > 0 {
            println!("      ✓ {}: fetched commits for {}/{} forks", 
                repo.slug(), forks_with_commits, repo.forks.len().min(MAX_FORKS_TO_PROCESS));
        }
    }

    Ok(repos)
}

fn print_summary(repos: &[Repo], language: &str) {
    println!("\n  Summary for {}:", language);
    println!("  - Total repositories: {}", repos.len());
    
    let total_stars: u64 = repos.iter().map(|r| r.stargazers_count).sum();
    let total_forks: u64 = repos.iter().map(|r| r.forks_count).sum();
    let total_issues: usize = repos.iter().map(|r| r.issues.len()).sum();
    let total_commits: usize = repos.iter().map(|r| r.recent_commits.len()).sum();
    
    println!("  - Total stars: {}", total_stars);
    println!("  - Total forks: {}", total_forks);
    println!("  - Total open issues: {}", total_issues);
    println!("  - Total commits fetched: {}", total_commits);
    
    let fork_commits: usize = repos.iter()
        .flat_map(|r| &r.forks)
        .map(|f| f.commit_count as usize)
        .sum();
    println!("  - Commits in forked repos: {}", fork_commits);
}
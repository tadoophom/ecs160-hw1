//! Data collection orchestration separated from business logic.
//! Handles the workflow of fetching data from GitHub API.

use crate::error::AppError;
use crate::model::Repo;
use crate::service::traits::GitRepositoryService;

/// Orchestrates data collection from any Git repository service
pub struct DataCollector<'a, S: GitRepositoryService> {
    service: &'a S,
}

impl<'a, S: GitRepositoryService> DataCollector<'a, S> {
    /// Creates a new data collector with any Git repository service
    pub fn new(service: &'a S) -> Self {
        Self { service }
    }

    /// Collects comprehensive data for repositories of a specific language
    pub async fn collect_language_data(&self, language: &str) -> Result<Vec<Repo>, AppError> {
        println!("  [1/4] Fetching top 10 repositories...");
        let mut repos = self.service.fetch_top_repositories(language, 10).await?;
        println!("      ✓ Found {} repositories", repos.len());

        println!("  [2/4] Fetching commits and issues for each repository...");
        self.enrich_repos_with_commits_and_issues(&mut repos).await;

        println!("  [3/4] Fetching forks for each repository...");
        self.enrich_repos_with_forks(&mut repos).await;

        println!("  [4/4] Fetching commits for forked repositories...");
        self.enrich_forks_with_commits(&mut repos).await;

        Ok(repos)
    }

    /// Enriches repositories with commit and issue data
    async fn enrich_repos_with_commits_and_issues(&self, repos: &mut [Repo]) {
        for repo in repos.iter_mut() {
            // Fetch commits
            match self.service
                .fetch_recent_commits(&repo.owner.login, &repo.name)
                .await
            {
                Ok(commits) => {
                    println!("      ✓ {}: {} commits", repo.slug(), commits.len());
                    repo.commit_count = commits.len() as u64;
                    
                    let mut detailed_commits = Vec::new();
                    for commit in commits.iter().take(50) { // MAX_COMMITS_WITH_FILES
                        match self.service
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

            // Fetch issues
            match self.service
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
    }

    /// Enriches repositories with fork data
    async fn enrich_repos_with_forks(&self, repos: &mut [Repo]) {
        for repo in repos.iter_mut() {
            match self.service
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
    }

    /// Enriches forks with commit data
    async fn enrich_forks_with_commits(&self, repos: &mut [Repo]) {
        for repo in repos.iter_mut() {
            for fork in repo.forks.iter_mut().take(20) { // MAX_FORKS_TO_PROCESS
                match self.service
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
                    repo.forks.len().min(20)
                );
            }
        }
    }
}

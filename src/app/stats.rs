//! Statistics calculation and metrics computation.
//! Handles calculation of repository metrics and language summaries.

use crate::app::{LanguageReport, RepoMetrics};
use crate::model::Repo;
use std::collections::HashMap;

/// Statistics calculator for repository data
pub struct StatsCalculator;

impl StatsCalculator {
    pub fn calculate_repo_stats(repos: &[Repo]) -> (Vec<RepoMetrics>, usize) {
        let mut metrics = Vec::with_capacity(repos.len());
        let mut fork_commit_total = 0usize;

        for repo in repos {
            let top_files = Self::get_top_files(repo);

            let new_fork_commits: usize = repo
                .forks
                .iter()
                .take(20) 
                .map(|fork| Self::count_new_commits(fork))
                .sum();

            fork_commit_total += new_fork_commits;

            metrics.push(RepoMetrics {
                slug: repo.slug(),
                top_files,
            });
        }

        (metrics, fork_commit_total)
    }

    fn count_new_commits(fork: &Repo) -> usize {
        let Some(fork_created_at) = &fork.created_at else {
            return 0; 
        };

        fork.recent_commits
            .iter()
            .filter(|commit| {
                commit
                    .commit
                    .author
                    .as_ref()
                    .and_then(|author| author.date.as_ref())
                    .map(|commit_date| commit_date > fork_created_at)
                    .unwrap_or(false)
            })
            .count()
    }

    fn get_top_files(repo: &Repo) -> Vec<String> {
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

    pub fn build_language_report(language: &str, repos: Vec<Repo>) -> LanguageReport {
        let total_stars: u64 = repos.iter().map(|r| r.stargazers_count).sum();
        let total_forks: u64 = repos.iter().map(|r| r.forks_count).sum();
        let total_open_issues: usize = repos.iter().map(|r| r.issues.len()).sum();
        let (repo_metrics, new_fork_commits) = Self::calculate_repo_stats(&repos);
        let total_repo_commits: usize = repos.iter().map(|r| r.commit_count as usize).sum();

        LanguageReport {
            language: language.to_string(),
            repos,
            total_stars,
            total_forks,
            total_open_issues,
            total_repo_commits,
            new_fork_commits,
            repo_metrics,
        }
    }
}

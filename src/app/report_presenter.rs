//! Report presentation logic separated from data collection and calculation.
//! Handles formatting and display of language reports.

use crate::app::LanguageReport;

/// Handles presentation and formatting of language reports
pub struct ReportPresenter;

impl ReportPresenter {
    /// Prints a formatted summary of a language report
    pub fn print_summary(report: &LanguageReport) {
        println!("Language: {}", report.language);
        println!("Total stars: {}", report.total_stars);
        println!("Total forks: {}", report.total_forks);
        println!("Top-3 Most modified file per repo:");
        for metrics in &report.repo_metrics {
            println!("  Repo name: {}", metrics.slug);
            if metrics.top_files.is_empty() {
                println!("    No files modified in recent commits");
            } else {
                for (idx, file) in metrics.top_files.iter().enumerate() {
                    println!("    File name{}: {}", idx + 1, file);
                }
            }
        }
        println!("New commits in forked repos: {}", report.new_fork_commits);
        println!("Open issues in top-10 repos: {}", report.total_open_issues);
    }
}

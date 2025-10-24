//! Output formatting and display logic.
//! Handles formatting and display of language reports.

use crate::app::LanguageReport;

pub struct OutputFormatter;

impl OutputFormatter {
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

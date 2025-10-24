//! Repository cloning.

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use crate::error::AppError;
use crate::model::Repo;

/// Rules to determine if a repository contains actual source code vs tutorials/documentation
#[derive(Debug, Clone)]
pub struct CodeDetectionRules {
    /// extensions that indicate source code
    pub source_extensions: HashSet<String>,
    /// Minimum ratio of source files to total files to consider it a code repo
    pub min_source_ratio: f64,
    /// Maximum directory depth to scan
    pub max_depth: usize,
}

impl CodeDetectionRules {
    /// Creates detection rules with provided parameters
    pub fn new(min_source_ratio: f64, max_depth: usize) -> Self {
        let extensions = [
            // Languages we're analyzing
            "java",
            "c",
            "cpp",
            "cc",
            "cxx",
            "h",
            "hpp",
            "rs",
            // Build/config files
            "cmake",
            "makefile",
            "gradle",
            "maven",
            "pom",
            "cargo",
            "toml",
            "xml",
            "properties",
            "json",
            "sh",
            "bat",
            // Additional C++ related files
            "hxx",
            "c++",
            "h++",
            "tcc",
            "tpp",
            "txx",
        ];

        let source_extensions = extensions.iter().map(|s| s.to_string()).collect();

        Self {
            source_extensions,
            min_source_ratio,
            max_depth,
        }
    }
}

impl Default for CodeDetectionRules {
    fn default() -> Self {
        Self::new(0.05, 10)
    }
}

/// Checks if a repository contains actual source code
pub fn check_for_source_code(
    repo_path: &Path,
    rules: &CodeDetectionRules,
) -> Result<CodeAnalysis, AppError> {
    let mut source_files = 0;
    let mut total_files = 0;
    let mut file_extensions: HashSet<String> = HashSet::new();

    if let Ok(entries) = walkdir::WalkDir::new(repo_path)
        .max_depth(rules.max_depth)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        for entry in entries {
            let path = entry.path();

            if path.is_file() {
                total_files += 1;

                if let Some(ext_str) = path.extension().and_then(|ext| ext.to_str()) {
                    let ext_lower = ext_str.to_lowercase();
                    file_extensions.insert(ext_lower.clone());

                    if rules.source_extensions.contains(&ext_lower) {
                        source_files += 1;
                    }
                }
            }
        }
    }

    let source_ratio = if total_files > 0 {
        source_files as f64 / total_files as f64
    } else {
        0.0
    };

    let is_source_code_repo = source_ratio >= rules.min_source_ratio && source_files > 0;

    Ok(CodeAnalysis {
        source_files,
        total_files,
        source_ratio,
        is_source_code_repo,
        file_extensions: file_extensions.into_iter().collect(),
    })
}

#[derive(Debug)]
pub struct CodeAnalysis {
    pub source_files: usize,
    pub total_files: usize,
    pub source_ratio: f64,
    pub is_source_code_repo: bool,
    pub file_extensions: Vec<String>,
}

pub async fn clone_repository(repo: &Repo, clone_dir: &Path) -> Result<(), AppError> {
    let clone_url = format!("https://github.com/{}.git", repo.slug());

    println!("  Cloning {} to {:?}...", repo.slug(), clone_dir);

    if let Some(parent) = clone_dir.parent() {
        std::fs::create_dir_all(parent).map_err(AppError::from)?;
    }

    let output = Command::new("git")
        .args(&[
            "clone",
            "--depth",
            "1",
            &clone_url,
            clone_dir.to_str().unwrap(),
        ])
        .output()
        .map_err(|_| {
            AppError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "git command not found. Please install git.",
            ))
        })?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Git(format!(
            "Failed to clone repository {}: {}",
            repo.slug(),
            error_msg
        )));
    }

    println!("  ✓ Successfully cloned {}", repo.slug());
    Ok(())
}

async fn clone_and_check_repo(
    repo: &Repo,
    clone_dir: &Path,
    rules: &CodeDetectionRules,
) -> Result<Option<(Repo, CodeAnalysis)>, AppError> {
    if let Err(e) = clone_repository(repo, clone_dir).await {
        eprintln!("    ⚠ Failed to clone {}: {}", repo.slug(), e);
        return Ok(None);
    }

    match check_for_source_code(clone_dir, rules) {
        Ok(analysis) => {
            println!(
                "    {}: {} source files, {:.1}% source ratio",
                repo.slug(),
                analysis.source_files,
                analysis.source_ratio * 100.0
            );

            if analysis.is_source_code_repo {
                println!(
                    "    ✓ {} appears to contain actual source code!",
                    repo.slug()
                );
                // Keep the cloned directory - don't clean up
                return Ok(Some((repo.clone(), analysis)));
            } else {
                println!("    ✗ {} appears to be documentation/tutorial", repo.slug());
            }
        }
        Err(e) => eprintln!("    ⚠ Failed to analyze {}: {}", repo.slug(), e),
    }

    if let Err(e) = std::fs::remove_dir_all(clone_dir) {
        eprintln!("    ⚠ Failed to clean up {}: {}", clone_dir.display(), e);
    }

    Ok(None)
}

pub async fn find_best_code_repo(
    repos: &[Repo],
    language: &str,
    clone_base_dir: &Path,
    min_source_ratio: f64,
) -> Result<Option<(Repo, CodeAnalysis)>, AppError> {
    let rules = CodeDetectionRules::new(min_source_ratio, 10);

    println!(
        "  Analyzing top {} repositories for source code content...",
        repos.len()
    );

    for (i, repo) in repos.iter().enumerate() {
        println!(
            "    [{}/{}] Checking {} ({} stars)...",
            i + 1,
            repos.len(),
            repo.slug(),
            repo.stargazers_count
        );

        let clone_dir = clone_base_dir.join(format!("{}-{}", language.to_lowercase(), repo.name));

        if let Ok(Some((repo_clone, analysis))) =
            clone_and_check_repo(repo, &clone_dir, &rules).await
        {
            println!(
                "    ✓ Found most popular source code repository: {} ({} stars)",
                repo.slug(),
                repo.stargazers_count
            );
            println!(
                "    ✓ Source files: {}, Source ratio: {:.1}%",
                analysis.source_files,
                analysis.source_ratio * 100.0
            );
            return Ok(Some((repo_clone, analysis)));
        } else {
            println!(
                "    ✗ {} appears to be documentation/tutorial only",
                repo.slug()
            );
        }
    }

    println!(
        "    ✗ No suitable source code repository found for {}",
        language
    );
    Ok(None)
}

/// Clones the best repo for each language and returns the list of cloned repos
pub async fn clone_best_repos(
    language_reports: &[crate::app::LanguageReport],
    clone_base_dir: &Path,
    min_source_ratio: f64,
) -> Result<Vec<Repo>, AppError> {
    println!("\n=== Part C: Clone and Inspect Repositories ===\n");

    let mut cloned_repos = Vec::new();

    for report in language_reports {
        println!("Processing {} repositories...", report.language);
        println!("{}", "=".repeat(50));

        match find_best_code_repo(
            &report.repos,
            &report.language,
            clone_base_dir,
            min_source_ratio,
        )
        .await
        {
            Ok(Some((repo, analysis))) => {
                println!(
                    "✓ Successfully cloned best source code repository for {}: {}",
                    report.language,
                    repo.slug()
                );
                println!("  - Stars: {}", repo.stargazers_count);
                println!("  - Source files: {}", analysis.source_files);
                println!("  - Source ratio: {:.1}%", analysis.source_ratio * 100.0);
                println!("  - File extensions: {:?}", analysis.file_extensions);
                cloned_repos.push(repo);
            }
            Ok(None) => {
                println!(
                    "✗ No suitable source code repository found for {}",
                    report.language
                );
            }
            Err(e) => {
                eprintln!(
                    "✗ Failed to process {} repositories: {}",
                    report.language, e
                );
            }
        }

        println!();
    }

    Ok(cloned_repos)
}

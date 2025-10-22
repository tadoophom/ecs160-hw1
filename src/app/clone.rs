//! Repository cloning and source code detection functionality.
//! Implements Part C of the assignment: clone repositories and determine which contain actual source code.

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use crate::error::AppError;
use crate::model::Repo;

/// Heuristics to determine if a repository contains actual source code vs tutorials/documentation
#[derive(Debug, Clone)]
pub struct SourceCodeHeuristics {
    /// File extensions that indicate source code
    pub source_extensions: HashSet<String>,
    /// Minimum ratio of source files to total files to consider it a code repo
    pub min_source_ratio: f64,
}

impl Default for SourceCodeHeuristics {
    fn default() -> Self {
        let mut source_extensions = HashSet::new();
        // Languages we're actually analyzing
        source_extensions.insert("java".to_string());
        source_extensions.insert("c".to_string());
        source_extensions.insert("cpp".to_string());
        source_extensions.insert("cc".to_string());
        source_extensions.insert("cxx".to_string());
        source_extensions.insert("h".to_string());
        source_extensions.insert("hpp".to_string());
        source_extensions.insert("rs".to_string());
        
        // Common build/config files that indicate real projects
        source_extensions.insert("cmake".to_string());
        source_extensions.insert("makefile".to_string());
        source_extensions.insert("gradle".to_string());
        source_extensions.insert("maven".to_string());
        source_extensions.insert("pom".to_string());
        source_extensions.insert("cargo".to_string());
        source_extensions.insert("toml".to_string());
        source_extensions.insert("xml".to_string());
        source_extensions.insert("properties".to_string());
        source_extensions.insert("yaml".to_string());
        source_extensions.insert("yml".to_string());
        source_extensions.insert("json".to_string());
        source_extensions.insert("sh".to_string());
        source_extensions.insert("bat".to_string());

        Self {
            source_extensions,
            min_source_ratio: 0.1, // At least 10% source files
        }
    }
}

/// Analyzes a repository to determine if it contains actual source code
pub fn analyze_repository_source_code(repo_path: &Path, heuristics: &SourceCodeHeuristics) -> Result<SourceCodeAnalysis, AppError> {
    let mut source_files = 0;
    let mut total_files = 0;
    let mut file_extensions: HashSet<String> = HashSet::new();

    // Walk through the repository directory
    if let Ok(entries) = std::fs::read_dir(repo_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                total_files += 1;
                
                if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                    let ext_lower = extension.to_lowercase();
                    file_extensions.insert(ext_lower.clone());
                    
                    if heuristics.source_extensions.contains(&ext_lower) {
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

    let is_source_code_repo = source_ratio >= heuristics.min_source_ratio && source_files > 0;

    Ok(SourceCodeAnalysis {
        source_files,
        total_files,
        source_ratio,
        is_source_code_repo,
        file_extensions: file_extensions.into_iter().collect(),
    })
}

#[derive(Debug)]
pub struct SourceCodeAnalysis {
    pub source_files: usize,
    pub total_files: usize,
    pub source_ratio: f64,
    pub is_source_code_repo: bool,
    pub file_extensions: Vec<String>,
}

/// Clones a repository to a local directory
pub async fn clone_repository(repo: &Repo, clone_dir: &Path) -> Result<(), AppError> {
    let clone_url = format!("https://github.com/{}.git", repo.slug());
    
    println!("  Cloning {} to {:?}...", repo.slug(), clone_dir);
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = clone_dir.parent() {
        std::fs::create_dir_all(parent).map_err(AppError::from)?;
    }

    // Clone with --depth 1 to only get the latest commit
    let output = Command::new("git")
        .args(&["clone", "--depth", "1", &clone_url, clone_dir.to_str().unwrap()])
        .output()
        .map_err(|_| AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "git command not found. Please install git."
        )))?;

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

/// Finds the most popular repository that contains actual source code for a given language
pub async fn find_best_source_code_repo(
    repos: &[Repo], 
    language: &str,
    clone_base_dir: &Path,
) -> Result<Option<(Repo, SourceCodeAnalysis)>, AppError> {
    let heuristics = SourceCodeHeuristics::default();
    
    println!("  Analyzing repositories for source code content...");
    
    for repo in repos {
        let clone_dir = clone_base_dir.join(format!("{}-{}", language.to_lowercase(), repo.name));
        
        // Clone the repository
        if let Err(e) = clone_repository(repo, &clone_dir).await {
            eprintln!("    ⚠ Failed to clone {}: {}", repo.slug(), e);
            continue;
        }
        
        // Analyze the repository
        match analyze_repository_source_code(&clone_dir, &heuristics) {
            Ok(analysis) => {
                println!("    {}: {} source files, {:.1}% source ratio",
                    repo.slug(),
                    analysis.source_files,
                    analysis.source_ratio * 100.0
                );
                
                if analysis.is_source_code_repo {
                    println!("    ✓ {} appears to contain actual source code!", repo.slug());
                    return Ok(Some((repo.clone(), analysis)));
                } else {
                    println!("    ✗ {} appears to be documentation/tutorial", repo.slug());
                }
            }
            Err(e) => {
                eprintln!("    ⚠ Failed to analyze {}: {}", repo.slug(), e);
            }
        }
        
        // Clean up the cloned directory
        if let Err(e) = std::fs::remove_dir_all(&clone_dir) {
            eprintln!("    ⚠ Failed to clean up {}: {}", clone_dir.display(), e);
        }
    }
    
    Ok(None)
}

/// Processes Part C: Clone and inspect repositories for each language
pub async fn process_repository_cloning(
    language_reports: &[crate::app::LanguageReport],
    clone_base_dir: &Path,
) -> Result<(), AppError> {
    println!("\n=== Part C: Clone and Inspect Repositories ===\n");
    
    for report in language_reports {
        println!("Processing {} repositories...", report.language);
        println!("{}", "=".repeat(50));
        
        match find_best_source_code_repo(&report.repos, &report.language, clone_base_dir).await {
            Ok(Some((repo, analysis))) => {
                println!("✓ Found best source code repository for {}: {}", 
                    report.language, repo.slug());
                println!("  - Stars: {}", repo.stargazers_count);
                println!("  - Source files: {}", analysis.source_files);
                println!("  - Source ratio: {:.1}%", analysis.source_ratio * 100.0);
                println!("  - File extensions: {:?}", analysis.file_extensions);
            }
            Ok(None) => {
                println!("✗ No suitable source code repository found for {}", report.language);
            }
            Err(e) => {
                eprintln!("✗ Failed to process {} repositories: {}", report.language, e);
            }
        }
        
        println!();
    }
    
    Ok(())
}

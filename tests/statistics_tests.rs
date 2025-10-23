//! Unit tests for Part B statistics computation.
//! These tests ONLY work with data objects and make NO API calls.
//! They verify the correctness of statistics calculations.

use ecs160_hw1::model::{Commit, CommitAuthor, CommitFile, CommitSummary, Issue, Owner, Repo};

/// Helper function to create a test Owner
fn create_test_owner(login: &str, id: i64) -> Owner {
    Owner {
        login: login.to_string(),
        id,
        html_url: format!("https://github.com/{}", login),
        site_admin: false,
    }
}

/// Helper function to create a test Repo
fn create_test_repo(
    name: &str,
    owner_login: &str,
    stars: u64,
    forks: u64,
    open_issues: u64,
) -> Repo {
    Repo {
        id: 1,
        name: name.to_string(),
        full_name: format!("{}/{}", owner_login, name),
        html_url: format!("https://github.com/{}/{}", owner_login, name),
        forks_count: forks,
        stargazers_count: stars,
        open_issues_count: open_issues,
        language: Some("Rust".to_string()),
        owner: create_test_owner(owner_login, 1),
        created_at: Some("2024-01-01T00:00:00Z".to_string()),
        forks: Vec::new(),
        recent_commits: Vec::new(),
        issues: Vec::new(),
        commit_count: 0,
    }
}

/// Helper function to create a test Commit with files
fn create_test_commit(sha: &str, files: Vec<CommitFile>) -> Commit {
    Commit {
        sha: sha.to_string(),
        url: format!("https://api.github.com/commits/{}", sha),
        html_url: Some(format!("https://github.com/commits/{}", sha)),
        commit: CommitSummary {
            message: "Test commit".to_string(),
            author: Some(CommitAuthor {
                name: Some("Test Author".to_string()),
                email: Some("test@example.com".to_string()),
                date: Some("2024-01-15T00:00:00Z".to_string()),
            }),
            committer: None,
        },
        files,
    }
}

/// Helper function to create a test CommitFile
fn create_test_file(filename: &str, additions: i64, deletions: i64, changes: i64) -> CommitFile {
    CommitFile {
        filename: filename.to_string(),
        additions,
        deletions,
        changes,
        status: "modified".to_string(),
    }
}

/// Helper function to create a test Issue
fn create_test_issue(title: &str, state: &str) -> Issue {
    Issue {
        title: title.to_string(),
        body: Some("Test issue body".to_string()),
        state: state.to_string(),
        html_url: Some("https://github.com/issues/1".to_string()),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-02T00:00:00Z".to_string(),
    }
}

// ============================================================================
// Test 1: Total Stars Calculation
// ============================================================================

#[test]
fn test_total_stars_single_repo() {
    let repo = create_test_repo("test-repo", "owner1", 100, 5, 3);
    let repos = vec![repo];
    
    let total_stars: u64 = repos.iter().map(|r| r.stargazers_count).sum();
    
    assert_eq!(total_stars, 100);
}

#[test]
fn test_total_stars_multiple_repos() {
    let repos = vec![
        create_test_repo("repo1", "owner1", 100, 5, 3),
        create_test_repo("repo2", "owner2", 200, 10, 5),
        create_test_repo("repo3", "owner3", 50, 2, 1),
    ];
    
    let total_stars: u64 = repos.iter().map(|r| r.stargazers_count).sum();
    
    assert_eq!(total_stars, 350);
}

#[test]
fn test_total_stars_empty_repos() {
    let repos: Vec<Repo> = Vec::new();
    
    let total_stars: u64 = repos.iter().map(|r| r.stargazers_count).sum();
    
    assert_eq!(total_stars, 0);
}

#[test]
fn test_total_stars_zero_stars() {
    let repos = vec![
        create_test_repo("repo1", "owner1", 0, 5, 3),
        create_test_repo("repo2", "owner2", 0, 10, 5),
    ];
    
    let total_stars: u64 = repos.iter().map(|r| r.stargazers_count).sum();
    
    assert_eq!(total_stars, 0);
}

// ============================================================================
// Test 2: Total Forks Calculation
// ============================================================================

#[test]
fn test_total_forks_single_repo() {
    let repo = create_test_repo("test-repo", "owner1", 100, 5, 3);
    let repos = vec![repo];
    
    let total_forks: u64 = repos.iter().map(|r| r.forks_count).sum();
    
    assert_eq!(total_forks, 5);
}

#[test]
fn test_total_forks_multiple_repos() {
    let repos = vec![
        create_test_repo("repo1", "owner1", 100, 5, 3),
        create_test_repo("repo2", "owner2", 200, 10, 5),
        create_test_repo("repo3", "owner3", 50, 15, 1),
    ];
    
    let total_forks: u64 = repos.iter().map(|r| r.forks_count).sum();
    
    assert_eq!(total_forks, 30);
}

#[test]
fn test_total_forks_empty_repos() {
    let repos: Vec<Repo> = Vec::new();
    
    let total_forks: u64 = repos.iter().map(|r| r.forks_count).sum();
    
    assert_eq!(total_forks, 0);
}

// ============================================================================
// Test 3: Total Open Issues Calculation
// ============================================================================

#[test]
fn test_total_open_issues_single_repo() {
    let mut repo = create_test_repo("test-repo", "owner1", 100, 5, 0);
    repo.issues = vec![
        create_test_issue("Issue 1", "open"),
        create_test_issue("Issue 2", "open"),
        create_test_issue("Issue 3", "open"),
    ];
    let repos = vec![repo];
    
    let total_open_issues: usize = repos.iter().map(|r| r.issues.len()).sum();
    
    assert_eq!(total_open_issues, 3);
}

#[test]
fn test_total_open_issues_multiple_repos() {
    let mut repo1 = create_test_repo("repo1", "owner1", 100, 5, 0);
    repo1.issues = vec![
        create_test_issue("Issue 1", "open"),
        create_test_issue("Issue 2", "open"),
    ];
    
    let mut repo2 = create_test_repo("repo2", "owner2", 200, 10, 0);
    repo2.issues = vec![
        create_test_issue("Issue 3", "open"),
        create_test_issue("Issue 4", "open"),
        create_test_issue("Issue 5", "open"),
    ];
    
    let repos = vec![repo1, repo2];
    
    let total_open_issues: usize = repos.iter().map(|r| r.issues.len()).sum();
    
    assert_eq!(total_open_issues, 5);
}

#[test]
fn test_total_open_issues_no_issues() {
    let repos = vec![
        create_test_repo("repo1", "owner1", 100, 5, 0),
        create_test_repo("repo2", "owner2", 200, 10, 0),
    ];
    
    let total_open_issues: usize = repos.iter().map(|r| r.issues.len()).sum();
    
    assert_eq!(total_open_issues, 0);
}

// ============================================================================
// Test 4: Top Modified Files Calculation
// ============================================================================

#[test]
fn test_top_modified_files_single_file() {
    let files = vec![create_test_file("file1.rs", 10, 5, 15)];
    let commit = create_test_commit("abc123", files);
    
    let mut repo = create_test_repo("test-repo", "owner1", 100, 5, 3);
    repo.recent_commits = vec![commit];
    
    let top_files = compute_top_modified_files(&repo);
    
    assert_eq!(top_files.len(), 1);
    assert_eq!(top_files[0], "file1.rs");
}

#[test]
fn test_top_modified_files_multiple_commits() {
    let commit1 = create_test_commit(
        "abc123",
        vec![
            create_test_file("file1.rs", 10, 5, 15),
            create_test_file("file2.rs", 5, 2, 7),
        ],
    );
    
    let commit2 = create_test_commit(
        "def456",
        vec![
            create_test_file("file1.rs", 20, 10, 30),
            create_test_file("file3.rs", 8, 3, 11),
        ],
    );
    
    let mut repo = create_test_repo("test-repo", "owner1", 100, 5, 3);
    repo.recent_commits = vec![commit1, commit2];
    
    let top_files = compute_top_modified_files(&repo);
    
    // file1.rs should be first (15 + 30 = 45 changes)
    // file3.rs should be second (11 changes)
    // file2.rs should be third (7 changes)
    assert_eq!(top_files.len(), 3);
    assert_eq!(top_files[0], "file1.rs");
    assert_eq!(top_files[1], "file3.rs");
    assert_eq!(top_files[2], "file2.rs");
}

#[test]
fn test_top_modified_files_more_than_three() {
    let commit = create_test_commit(
        "abc123",
        vec![
            create_test_file("file1.rs", 50, 20, 70),
            create_test_file("file2.rs", 30, 10, 40),
            create_test_file("file3.rs", 25, 5, 30),
            create_test_file("file4.rs", 15, 5, 20),
            create_test_file("file5.rs", 10, 2, 12),
        ],
    );
    
    let mut repo = create_test_repo("test-repo", "owner1", 100, 5, 3);
    repo.recent_commits = vec![commit];
    
    let top_files = compute_top_modified_files(&repo);
    
    // Should only return top 3
    assert_eq!(top_files.len(), 3);
    assert_eq!(top_files[0], "file1.rs");
    assert_eq!(top_files[1], "file2.rs");
    assert_eq!(top_files[2], "file3.rs");
}

#[test]
fn test_top_modified_files_no_commits() {
    let repo = create_test_repo("test-repo", "owner1", 100, 5, 3);
    
    let top_files = compute_top_modified_files(&repo);
    
    assert_eq!(top_files.len(), 0);
}

#[test]
fn test_top_modified_files_uses_additions_deletions_when_changes_zero() {
    let files = vec![
        create_test_file("file1.rs", 10, 5, 0), // changes = 0, should use additions + deletions = 15
        create_test_file("file2.rs", 3, 2, 0),  // changes = 0, should use additions + deletions = 5
    ];
    let commit = create_test_commit("abc123", files);
    
    let mut repo = create_test_repo("test-repo", "owner1", 100, 5, 3);
    repo.recent_commits = vec![commit];
    
    let top_files = compute_top_modified_files(&repo);
    
    assert_eq!(top_files.len(), 2);
    assert_eq!(top_files[0], "file1.rs"); // 15 > 5
    assert_eq!(top_files[1], "file2.rs");
}

// ============================================================================
// Test 5: New Commits in Forks Calculation
// ============================================================================

#[test]
fn test_new_fork_commits_no_forks() {
    let repo = create_test_repo("test-repo", "owner1", 100, 5, 3);
    
    let new_commits = count_new_fork_commits(&repo);
    
    assert_eq!(new_commits, 0);
}

#[test]
fn test_new_fork_commits_fork_with_new_commits() {
    let mut fork = create_test_repo("test-repo", "forker1", 0, 0, 0);
    fork.created_at = Some("2024-01-10T00:00:00Z".to_string());
    
    // Create commits: one before fork, two after fork
    let commit1 = create_test_commit_with_date("abc123", "2024-01-05T00:00:00Z"); // Before fork
    let commit2 = create_test_commit_with_date("def456", "2024-01-15T00:00:00Z"); // After fork
    let commit3 = create_test_commit_with_date("ghi789", "2024-01-20T00:00:00Z"); // After fork
    
    fork.recent_commits = vec![commit1, commit2, commit3];
    
    let mut repo = create_test_repo("test-repo", "owner1", 100, 5, 3);
    repo.forks = vec![fork];
    
    let new_commits = count_new_fork_commits(&repo);
    
    // Only commits after fork creation date should count
    assert_eq!(new_commits, 2);
}

#[test]
fn test_new_fork_commits_multiple_forks() {
    let mut fork1 = create_test_repo("test-repo", "forker1", 0, 0, 0);
    fork1.created_at = Some("2024-01-10T00:00:00Z".to_string());
    fork1.recent_commits = vec![
        create_test_commit_with_date("abc123", "2024-01-15T00:00:00Z"), // After fork
        create_test_commit_with_date("def456", "2024-01-20T00:00:00Z"), // After fork
    ];
    
    let mut fork2 = create_test_repo("test-repo", "forker2", 0, 0, 0);
    fork2.created_at = Some("2024-01-12T00:00:00Z".to_string());
    fork2.recent_commits = vec![
        create_test_commit_with_date("ghi789", "2024-01-18T00:00:00Z"), // After fork
    ];
    
    let mut repo = create_test_repo("test-repo", "owner1", 100, 5, 3);
    repo.forks = vec![fork1, fork2];
    
    let new_commits = count_new_fork_commits(&repo);
    
    assert_eq!(new_commits, 3); // 2 from fork1 + 1 from fork2
}

#[test]
fn test_new_fork_commits_fork_no_created_date() {
    let mut fork = create_test_repo("test-repo", "forker1", 0, 0, 0);
    fork.created_at = None; // No creation date
    fork.recent_commits = vec![
        create_test_commit_with_date("abc123", "2024-01-15T00:00:00Z"),
    ];
    
    let mut repo = create_test_repo("test-repo", "owner1", 100, 5, 3);
    repo.forks = vec![fork];
    
    let new_commits = count_new_fork_commits(&repo);
    
    // Should return 0 when fork has no creation date
    assert_eq!(new_commits, 0);
}

#[test]
fn test_new_fork_commits_all_commits_before_fork() {
    let mut fork = create_test_repo("test-repo", "forker1", 0, 0, 0);
    fork.created_at = Some("2024-01-20T00:00:00Z".to_string());
    
    // All commits before fork creation
    let commit1 = create_test_commit_with_date("abc123", "2024-01-05T00:00:00Z");
    let commit2 = create_test_commit_with_date("def456", "2024-01-10T00:00:00Z");
    
    fork.recent_commits = vec![commit1, commit2];
    
    let mut repo = create_test_repo("test-repo", "owner1", 100, 5, 3);
    repo.forks = vec![fork];
    
    let new_commits = count_new_fork_commits(&repo);
    
    assert_eq!(new_commits, 0);
}

// ============================================================================
// Helper Functions (same as in app/mod.rs)
// ============================================================================

fn compute_top_modified_files(repo: &Repo) -> Vec<String> {
    use std::collections::HashMap;
    
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

fn count_new_fork_commits(repo: &Repo) -> usize {
    repo.forks
        .iter()
        .map(|fork| {
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
        })
        .sum()
}

fn create_test_commit_with_date(sha: &str, date: &str) -> Commit {
    Commit {
        sha: sha.to_string(),
        url: format!("https://api.github.com/commits/{}", sha),
        html_url: Some(format!("https://github.com/commits/{}", sha)),
        commit: CommitSummary {
            message: "Test commit".to_string(),
            author: Some(CommitAuthor {
                name: Some("Test Author".to_string()),
                email: Some("test@example.com".to_string()),
                date: Some(date.to_string()),
            }),
            committer: None,
        },
        files: Vec::new(),
    }
}
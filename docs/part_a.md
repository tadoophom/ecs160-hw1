# Part A – Fetching GitHub Repository Data

This document explains the problem requirements for Part A of the assignment and how the current Rust implementation addresses them.

## Problem recap
- Query the GitHub REST API for the top 10 most popular repositories written in specific languages (Java, C / C++, Rust).
- For each repository, also retrieve
  - the list of forks,
  - the most recent commits for the base repository and for each fork,
  - the open issues.
- Parse the JSON responses into strongly typed Rust structures so later stages can compute statistics.

## High-level flow in the code

### Entry point
`src/main.rs` wires Tokio’s async runtime to `app::run()`, which orchestrates the assignment workflow.

```rust
#[tokio::main]
async fn main() {
    if let Err(err) = ecs160_hw1::app::run().await {
        eprintln!("application error: {err}");
        std::process::exit(1);
    }
}
```

### Configuration loading
- `AppConfig::load()` (`src/config.rs`) pulls GitHub configuration from environment variables or `.env`.
- `GitHubConfig` keeps the API base URL, optional token, and user agent header.

### Fetching repositories
- `app::run()` iterates over `TARGET_LANGUAGES` (`["Java", "C", "C++", "Rust"]`).
- For each language, it calls `collect_language_report(&service, language)`.
- `GitService::fetch_top_repositories` (`src/service/git_service.rs`) uses `reqwest` to call the `/search/repositories` endpoint with the query `language:<lang>`, sorted by stars.
- Responses are parsed into `model::Repo` objects via `Repo::from_json`. That helper uses utility functions in `src/util/json.rs` to provide type-safe extraction with meaningful error messages.

### Fetching commit histories
- For each top repository, `collect_language_report` calls `GitService::fetch_recent_commits`.
  - This hits `/repos/{owner}/{repo}/commits` (first 50 entries).
  - The response is parsed into lightweight `model::Commit` values (without file diffs).
- The code then refines up to 50 of those commits by calling `GitService::fetch_commit_with_files` for each SHA, giving file-level change data needed later for Part B.

### Fetching open issues
- `GitService::fetch_open_issues` targets `/repos/{owner}/{repo}/issues?state=open`.
- The resulting array is converted into `model::Issue` structs.

### Fetching forks and their commits
- `GitService::fetch_repo_forks` requests `/repos/{owner}/{repo}/forks`, sorted by newest, up to 100 entries.
- Each fork is represented as another `Repo`.
- `collect_language_report` walks up to 20 forks per repo and calls `fetch_recent_commits` on each fork to gather their latest activity.

### Data collected for later steps
For every repository, the following fields are populated:
- `Repo::forks`: the list of fork metadata.
- `Repo::recent_commits`: detailed commits (with changed files) for the base repo.
- `Repo::issues`: open issues fetched from GitHub.
- `Repo::commit_count`: number of recent commits fetched for the base repo.
- For each fork, `Repo::recent_commits` and `Repo::commit_count` capture the fork’s commits so new fork activity can be counted in Part B.

## Notable Rust concepts in Part A
- **Async/Await & Tokio**: `#[tokio::main]` and `async fn` allow concurrent HTTP requests without blocking.
- **Structs and Traits**: `GitService`, `Repo`, `Commit`, `Issue`, and `Owner` are plain Rust structs. Traits like `ConfigSource` abstract configuration input.
- **Error Handling**: The project defines a custom `AppError` enum and returns `Result<T, AppError>` (aliased as `AppResult<T>`).
- **JSON Parsing with serde_json**: Manual parsing functions ensure the app gracefully handles missing or malformed fields.

Part A is largely implemented: the program gathers repositories, commits, issues, and forks for each target language and stores the results in strongly typed structures ready for analysis.

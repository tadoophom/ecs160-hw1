# Part C – Cloning and Inspecting Repositories

This document outlines the expectations for Part C and walks through the Rust implementation that attempts to satisfy them.

## Problem recap
1. Decide which of the top 10 repositories per language actually contain source code rather than tutorials or documentation.
2. Clone the most popular repository for each language that appears to be a real codebase (shallow clone is acceptable).
3. Provide evidence that the cloned repository contains source code.

## Implementation tour

### Workflow entry point
- `app::run()` (see `src/app/mod.rs`) delegates to `clone::process_repository_cloning`.
- It passes the collected `LanguageReport` data and a base directory (`./cloned_repos`) where clones should live.

### Cloning orchestration (`process_repository_cloning`)
- Iterates over each language’s report.
- Calls `find_best_source_code_repo`, which runs the heuristic inspection in descending popularity order.
- Prints the winner’s statistics (stars, source file count, ratio, file extensions) or a failure message.

### Selecting the “best” repository (`find_best_source_code_repo`)
- Uses `SourceCodeHeuristics::default()` to define which file extensions count as source: Java, C/C++, Rust, plus several build/config formats.
- For each repository:
  1. Builds a per-language clone directory name (e.g., `cloned_repos/java-repo-name`).
  2. Calls `clone_and_analyze_repo`.
  3. Returns the first repo that satisfies the heuristic (`analysis.is_source_code_repo`).

### Cloning (`clone_repository`)
- Shells out to the `git` CLI: `git clone --depth 1 https://github.com/{owner}/{repo}.git`.
- Ensures the parent directory exists and reports success/failure.

### Source detection (`analyze_repository_source_code`)
- Scans the cloned directory with `std::fs::read_dir`.
- Counts files whose lowercase extension is in the `SourceCodeHeuristics::source_extensions` set.
- Computes a ratio (`source_files / total_files`) and marks the repo as code if the ratio is at least 10% and at least one source file was seen.
- Collects all unique extensions encountered for reporting.

### Cleanup
- `clone_and_analyze_repo` removes the clone directory (`std::fs::remove_dir_all`) when the repository fails the heuristic, so disk usage stays contained. Successful matches are kept so the user can inspect them manually.

## Rust concepts in play
- **Async + blocking interop**: Cloning still relies on blocking `std::process::Command`. Tokio’s runtime can handle this, but spawning a blocking task would be cleaner for long-running clones.
- **HashSet / HashMap**: `HashSet` tracks extensions and prevents duplicates.
- **Error bubbling**: All helper functions return `Result<_, AppError>` so `?` can propagate errors up to `process_repository_cloning`.

## Current limitations and considerations
- **Shallow file scan**: `analyze_repository_source_code` only looks at the immediate entries returned by `read_dir` (non-recursive). Nested `src/` directories are ignored, so real projects that keep code in subdirectories often look like documentation-only repos. A recursive walk (`WalkDir`, manual DFS, etc.) is needed for accuracy.
- **Extension coverage**: Files such as `Makefile` or `BUILD` lack extensions, so the heuristic misses common build files even though the intent was to include them.
- **Clone directory reuse**: If a clone directory already exists (e.g., from a previous run), `git clone` fails because it refuses to overwrite the directory. Removing or reusing existing directories would make re-runs smoother.
- **Verification signal**: Besides printing counts, the program does not perform deeper checks (e.g., verifying the presence of package manifests or compilation targets). Depending on grader expectations, additional evidence might be required.

Despite these gaps, the Part C module demonstrates the requested cloning workflow and lays groundwork for more robust source detection.

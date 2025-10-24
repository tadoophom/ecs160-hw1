# ECS160 HW1 - GitHub Repository Analyzer

A Rust program that analyzes popular GitHub repositories across different programming languages using the GitHub REST API.

## What This Project Does

This program fetches data about the top 10 most popular repositories for Java, C/C++, and Rust from GitHub. It calculates various statistics about these repositories, clones the ones that contain actual source code, and stores everything in Redis for later use.

## Setup Instructions

### Prerequisites

You'll need these installed before running:
- Rust (1.70 or later) - https://www.rust-lang.org/tools/install
- Git 
- Redis server
- A GitHub Personal Access Token

### Installing Redis

On macOS:
```bash
brew install redis
brew services start redis
```

On Ubuntu/Debian:
```bash
sudo apt-get install redis-server
sudo systemctl start redis
```

Check if Redis is running:
```bash
redis-cli ping
# Should return "PONG"
```

### Getting a GitHub Token

You need a GitHub Personal Access Token to avoid API rate limits.

1. Go to https://github.com/settings/tokens
2. Click "Generate new token" → "Generate new token (classic)"
3. Give it a name like "ECS160-HW1"
4. Under scopes, check **public_repo**
5. Click "Generate token" and copy it immediately

### Setting Up the .env File

Create a file called `.env` in the project root:

```bash
cd ecs160-hw1
touch .env
```

Add this to the `.env` file:
```
GITHUB_TOKEN=paste_your_token_here
REDIS_URL=redis://127.0.0.1:6379
```

**Important:** Don't commit the `.env` file to git! It's already in the `.gitignore` file.

### Building the Project

```bash
cargo build --release
```

## Running the Program

Make sure Redis is running first!

```bash
cargo run
```

The program will:
1. Fetch the top 10 repos for each language (Java, C/C++, Rust)
2. Get commits, issues, and fork information for each repo
3. Calculate statistics
4. Clone repositories that contain actual source code (saved to `./cloned_repos/`)
5. Store everything in Redis

## Running Tests

```bash
cargo test
```

All unit tests use mock data and don't call the GitHub API.

## Design Decisions and Assumptions

Here are the assumptions I made for this assignment:

### Part A - Data Collection
- Getting the top 10 repos by searching GitHub API sorted by stars
- Fetching the first 50 commits per repository (to avoid too many API calls)
- Processing only the first 20 forks per repo for commit analysis
- Grabbing the first 100 open issues per repo

### Part B - Statistics
- "New commits" in forks means commits made AFTER the fork was created
- Comparing commit dates using the author date field from the GitHub API
- Top 3 modified files are based on total changes (additions + deletions) across recent commits

### Part C - Source Code Detection
I used a simple heuristic to determine if a repo contains actual source code:
- Count files with source extensions: .java, .c, .cpp, .rs, .h, .hpp, etc.
- Also count build files: .toml, .xml, .gradle, Makefile
- If at least 5% of files are source files and there's at least 1 source file, it's a code repo
- Scanning up to 3 directory levels deep

The program clones the most popular (by stars) repo that passes this check for each language.

### Part D - Redis Storage
Data is stored using these key patterns:
- Repositories: `repo:{owner}:{name}`
- Authors: `author:{login}`
- Issues: `issue:{repo_id}:{index}`

### Error Handling
- If one repo fails to fetch, the program continues with the others
- Missing optional fields are treated as empty/default values
- If a clone fails, it continues with other languages

## Project Structure

```
src/
├── main.rs              # Entry point
├── lib.rs               # Library exports
├── config.rs            # Configuration from .env
├── error.rs             # Error types
├── app/                 # Main application logic
│   ├── mod.rs           # Workflow coordinator
│   ├── clone.rs         # Repo cloning
│   ├── output.rs        # Result formatting
│   ├── repo_fetcher.rs  # Data fetching
│   └── stats.rs         # Statistics calculation
├── model/               # Data structures
│   ├── repo.rs
│   ├── commit.rs
│   ├── issue.rs
│   └── owner.rs
├── service/             # External services
│   ├── git_service.rs   # GitHub API client
│   ├── redis_service.rs # Redis storage
│   ├── traits.rs        # Service abstractions
│   └── test_services.rs # Mocks for testing
└── util/
    └── json.rs          # JSON parsing helpers
```

## Dependencies

Main libraries used:
- `reqwest` - HTTP client for GitHub API
- `tokio` - Async runtime
- `serde` / `serde_json` - JSON parsing
- `redis` - Redis client
- `chrono` - Date/time handling
- `walkdir` - Directory traversal for clone inspection
- `httpmock` - HTTP mocking for tests

## Example Output

```
=== Part A: Fetching GitHub Repository Data ===

Processing language: Java
==================================================
  [1/4] Fetching top 10 repositories...
      ✓ Found 10 repositories
  [2/4] Fetching commits and issues for each repository...
      ✓ spring-projects/spring-boot: 50 commits
      ✓ spring-projects/spring-boot: 245 open issues
  [3/4] Fetching forks for each repository...
      ✓ spring-projects/spring-boot: 100 forks
  [4/4] Fetching commits for forked repositories...
      ✓ spring-projects/spring-boot: fetched commits for 20/20 forks

Language: Java
Total stars: 125430
Total forks: 45678
Top-3 Most modified file per repo:
  Repo name: spring-projects/spring-boot
    File name1: pom.xml
    File name2: src/main/java/Application.java
    File name3: README.md
New commits in forked repos: 342
Open issues in top-10 repos: 1205

=== Part C: Clone and Inspect Repositories ===

Processing Java repositories...
==================================================
  Analyzing top 10 repositories for source code content...
    [1/10] Checking spring-projects/spring-boot (75234 stars)...
  ✓ Successfully cloned spring-projects/spring-boot
    spring-projects/spring-boot: 1247 source files, 89.3% source ratio
    ✓ Contains actual source code!

=== Part D: Storing Results in Redis ===

✓ Successfully stored all repository data in Redis
```

## Submission Checklist

Before submitting, make sure:
-  `.env` file is NOT committed to git (it's in `.gitignore`)
-  No API tokens or keys are hardcoded in the source code
-  `cargo run` works correctly
-  `cargo test` passes all tests
-  Redis is installed and running
-  The `cloned_repos/` directory is NOT included in the zip file
-  GitHub Actions is set up for CI/CD

## Important Notes for Grading

**Security:** 
- My GitHub token is only in the `.env` file, never in the code
- The `.env` file is in `.gitignore` so it won't be committed

**Cloned Repositories:**
- The program creates a `cloned_repos/` directory when it runs
- This directory is NOT included in my submission zip

**Execution:**
- Run with: `cargo run`
- Tests with: `cargo test`
- Redis must be running before starting the program

## Known Limitations

- Only fetches first 100 issues per repo (GitHub API pagination)
- Processes maximum 20 forks per repo for performance
- Gets detailed file info for first 50 commits only
- Source code detection is heuristic-based (might miss some edge cases)

## Testing

All unit tests follow the Part E requirements:
- Tests only pass Rust objects to functions
- No GitHub API calls during testing
- Uses mock services instead of real HTTP requests
- GitHub Actions runs tests automatically on every push

## Troubleshooting

**"Connection refused" error:**
- Make sure Redis is running: `redis-cli ping`
- Start Redis if needed: `brew services start redis` (macOS) or `sudo systemctl start redis` (Linux)

**API rate limit errors:**
- Check that GITHUB_TOKEN is set correctly in `.env`
- Authenticated requests get 5000/hour vs 60/hour without auth

**Clone failures:**
- Make sure `git` is installed
- Check your internet connection
- Some repos might be very large and take time to clone
# ecs160-hw1

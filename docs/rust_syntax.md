# Rust Syntax Primer for This Project

This short guide highlights key Rust language features that appear throughout the project so you can read and extend the code with confidence.

## Async entry point
- `#[tokio::main]` turns `main` into an async function backed by the Tokio runtime.
- `async fn main() { ... }` returns a `Future`; Tokio drives it to completion.
- `if let Err(err) = ... { ... }` pattern-matches a `Result`, letting you handle only the `Err` case.

```rust
#[tokio::main]
async fn main() {
    if let Err(err) = ecs160_hw1::app::run().await {
        eprintln!("application error: {err}");
        std::process::exit(1);
    }
}
```

## Modules and namespaces
- `pub mod app;` in `lib.rs` exposes the `app` module to other crates.
- Inside modules, `use crate::service::GitService;` brings items into scope.
- Files like `src/app/mod.rs` define the contents of a module; sibling files (e.g., `clone.rs`) become submodules via `pub mod clone;`.

## Structs and implementations
- Structs group related data:

```rust
#[derive(Debug, Clone)]
pub struct Repo {
    pub id: i64,
    pub name: String,
    pub forks_count: u64,
    // ...
}
```

- `impl Repo { ... }` adds methods associated with the struct. `pub fn slug(&self) -> String` borrows `self` immutably (`&self`) and returns a `String`.

## Results and error handling
- Most fallible functions return `Result<T, AppError>` (type alias `AppResult<T>` in `lib.rs`).
- The `?` operator propagates errors: `let config = AppConfig::load()?;`.
- `thiserror::Error` derives enhance error enums with readable messages.

## Pattern matching & control flow
- `if let` and `match` handle complex branching on enums and options.
- Example: extracting commit dates safely:

```rust
commit
    .commit
    .author
    .as_ref()
    .and_then(|author| author.date.as_ref())
    .map(|commit_date| commit_date > fork_created_at)
    .unwrap_or(false)
```

- `as_ref()` converts `Option<T>` to `Option<&T>`, enabling chained borrowing without moving the data.

## Collections and iterators
- `Vec<T>` for ordered lists, `HashMap` / `HashSet` for keyed data.
- Iterators (from `iter()`) support functional-style transforms:
  - `.map(|r| r.stargazers_count).sum()` accumulates totals.
  - `.filter(...)` keeps only matching items.
  - `.collect::<Result<Vec<_>, _>>()?` gathers results while short-circuiting on errors.

## String formatting
- `format!("{}/{}", self.owner.login, self.name)` mirrors `printf`-style formatting.
- `println!` and `eprintln!` are macros (note the `!`) for writing to stdout and stderr.

## Borrowing vs. ownership
- Function parameters use references (e.g., `service: &GitService`) to avoid unnecessary cloning.
- `Repo` derives `Clone`, letting the code duplicate repository data when needed (e.g., returning a cloned repo from Part C).
- Slices (`&[Repo]`) allow read-only access to contiguous data without copying.

## Standard library utilities
- `std::collections::HashMap` supplies hash-based maps.
- `std::process::Command` runs external programs (like `git clone`).
- `std::path::Path` and `Path::join` build filesystem paths in an OS-neutral way.

Keep this sheet handy as you explore the code; it covers the recurring syntax patterns and library traits used across the project.

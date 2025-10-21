//! Domain models for repos, commits, issues, and related types.
//! Each submodule owns its JSON parsing logic so higher layers get typed data.
pub mod commit;
pub mod issue;
pub mod owner;
pub mod repo;

pub use commit::{Commit, CommitAuthor, CommitFile, CommitSummary};
pub use issue::Issue;
pub use owner::Owner;
pub use repo::Repo;

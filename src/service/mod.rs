//! Service layer providing abstractions for external dependencies.
//! Implements SOLID principles through trait-based design.
pub mod git_service;
pub mod interfaces;
pub mod redis_service;
pub mod test_services;
pub mod traits;

pub use git_service::GitService;
pub use redis_service::RedisService;
pub use test_services::{TestGitService, TestStorageService};
pub use traits::*;
// Note: interfaces provides additional specialized interfaces

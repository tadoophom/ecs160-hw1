//! Service layer providing abstractions for external dependencies.
//! Implements SOLID principles through trait-based design.
pub mod git_service;
pub mod redis_service;
pub mod traits;
pub mod test_services;
pub mod interfaces;

pub use git_service::GitService;
pub use redis_service::RedisService;
pub use traits::*;
pub use test_services::{TestGitService, TestStorageService};
// Note: interfaces provides additional specialized interfaces

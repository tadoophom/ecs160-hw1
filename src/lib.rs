//! Crate root exposing the application modules and shared exports.
//! Provides shared helpers such as the GitHub service facade and result alias.
pub mod app;
pub mod config;
pub mod error;
pub mod model;
pub mod service;
pub mod util;

/// Shared result type alias for fallible operations within the application.
pub type AppResult<T> = Result<T, error::AppError>;

pub use service::git_service::GitService;

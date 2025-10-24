//! Application modules.
pub mod app;
pub mod config;
pub mod error;
pub mod model;
pub mod service;
pub mod util;

pub type AppResult<T> = Result<T, error::AppError>;

pub use service::git_service::GitService;

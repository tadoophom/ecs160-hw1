pub mod config;
pub mod error;
pub mod github;
pub mod models;

/// Shared result type alias for fallible operations within the application.
pub type AppResult<T> = Result<T, error::AppError>;

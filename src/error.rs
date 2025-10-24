//! Error types.
use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("missing GitHub token in configuration")]
    MissingGitHubToken,
    #[error("http client error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("github api error: {0}")]
    GitHubApi(String),
    #[error("git operation error: {0}")]
    Git(String),
    #[error("redis error: {0}")]
    Redis(String),
    #[error("feature not implemented yet")]
    NotImplemented,
}

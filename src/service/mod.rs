//! Houses service-layer components used by higher-level workflows.
//! Currently exports the GitHub HTTP facade consumed by the app layer.
pub mod git_service;

pub use git_service::GitService;

use std::path::PathBuf;

use thiserror::Error;

/**
    An error that may occur when interacting with Roblox Studio.
*/
#[derive(Debug, Error)]
pub enum RobloxStudioError {
    #[error("Unknown task: {0}")]
    UnknownTask(String),
    #[error("Failed to find user documents directory")]
    UserDocumentsDirMissing,
    #[error("Failed to find local data directory")]
    LocalDataDirMissing,
    #[error("Failed to copy place file to local data directory: {0}")]
    LocalDataDirCopyPlace(String),
    #[error("Failed to canonicalize path: {0}")]
    PathCanonicalize(String),
    #[error("Failed to convert path to string: {0:?}")]
    PathToString(PathBuf),
    #[error("Failed to open Roblox Studio: {0}")]
    Opener(#[from] opener::OpenError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/**
    Type alias for results that return a `RobloxStudioError`.
*/
pub type RobloxStudioResult<T> = Result<T, RobloxStudioError>;

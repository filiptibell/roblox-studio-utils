use std::{env, error::Error, fmt, io, path::PathBuf};

/**
    An error that may occur when interacting with Roblox Studio.
*/
#[derive(Debug)]
#[non_exhaustive]
pub enum RobloxStudioError {
    UnsupportedPlatform,
    UnknownTask(String),
    UserDocumentsDirMissing,
    LocalDataDirMissing,
    LocalDataDirCopyPlace(String),
    PathCanonicalize(String),
    PathToString(PathBuf),
    Io(io::Error),
}

impl fmt::Display for RobloxStudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RobloxStudioError::UnsupportedPlatform => {
                write!(f, "unsupported platform: {}", env::consts::OS)
            }
            RobloxStudioError::UnknownTask(s) => write!(f, "unknown task: {s}"),
            RobloxStudioError::UserDocumentsDirMissing => {
                write!(f, "failed to find user documents directory")
            }
            RobloxStudioError::LocalDataDirMissing => {
                write!(f, "failed to find local data directory")
            }
            RobloxStudioError::LocalDataDirCopyPlace(s) => {
                write!(f, "failed to copy place file to local data directory: {s}")
            }
            RobloxStudioError::PathCanonicalize(s) => write!(f, "failed to canonicalize path: {s}"),
            RobloxStudioError::PathToString(p) => {
                write!(f, "failed to convert path to string: {}", p.display())
            }
            RobloxStudioError::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl Error for RobloxStudioError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::Io(e) = self {
            Some(e)
        } else {
            None
        }
    }
}

impl From<io::Error> for RobloxStudioError {
    fn from(value: io::Error) -> Self {
        RobloxStudioError::Io(value)
    }
}

/**
    Type alias for results that return a `RobloxStudioError`.
*/
pub type RobloxStudioResult<T> = Result<T, RobloxStudioError>;

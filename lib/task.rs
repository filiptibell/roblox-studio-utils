use std::{ffi::OsString, fmt, str::FromStr};

use crate::RobloxStudioError;

/**
    A task that can be performed by Roblox Studio.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RobloxStudioTask {
    EditPlace,
    EditFile,
    StartServer,
    StartClient,
}

impl RobloxStudioTask {
    /**
        Tries to parse a task from a string.

        This is case insensitive and also accepts optional
        separators for cases like `edit-place` or `edit_place`.
    */
    #[must_use]
    pub fn parse(s: impl AsRef<str>) -> Option<Self> {
        match s.as_ref().trim().to_ascii_lowercase().as_str() {
            "editplace" | "edit-place" | "edit_place" => Some(Self::EditPlace),
            "editfile" | "edit-file" | "edit_file" => Some(Self::EditFile),
            "startserver" | "start-server" | "start_server" => Some(Self::StartServer),
            "startclient" | "start-client" | "start_client" => Some(Self::StartClient),
            _ => None,
        }
    }

    /**
        Returns the name of the task.

        This is always a PascalCase string.
    */
    #[must_use]
    #[allow(clippy::doc_markdown)]
    pub const fn name(self) -> &'static str {
        match self {
            Self::EditPlace => "EditPlace",
            Self::EditFile => "EditFile",
            Self::StartServer => "StartServer",
            Self::StartClient => "StartClient",
        }
    }
}

impl FromStr for RobloxStudioTask {
    type Err = RobloxStudioError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| RobloxStudioError::UnknownTask(s.to_string()))
    }
}

impl fmt::Display for RobloxStudioTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl From<RobloxStudioTask> for OsString {
    fn from(value: RobloxStudioTask) -> Self {
        value.to_string().into()
    }
}

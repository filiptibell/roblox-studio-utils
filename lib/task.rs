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
    RunScript,
}

impl RobloxStudioTask {
    /**
        Certain tasks only work using the `RobloxStudioBetaLauncher`
        executable on Windows, and fail with a permission error
        when using the plain `RobloxStudioBeta` executable.

        This method returns `true` when the launcher must be used.
    */
    #[must_use]
    pub const fn needs_launcher(self) -> bool {
        matches!(self, Self::EditPlace)
    }

    /**
        Certain tasks start, or connect to, a local test server,
        and need to be given a server address and port.

        This method returns `true` when the task uses a server.
    */
    #[must_use]
    pub const fn uses_server(self) -> bool {
        matches!(self, Self::StartServer | Self::StartClient)
    }

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
            "runscript" | "run-script" | "run_script" => Some(Self::RunScript),
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
            Self::RunScript => "RunScript",
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
        value.name().into()
    }
}

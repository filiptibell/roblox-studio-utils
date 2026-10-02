use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::RobloxStudioResult;
use crate::task::RobloxStudioTask;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

/**
    References to discovered paths to the current Roblox Studio
    executable, content, plugins, and settings directories.

    The executable is guaranteed to exist.

    Can be cheaply cloned and shared between threads.
*/
#[derive(Debug, Clone)]
pub struct RobloxStudioPaths {
    inner: Arc<RobloxStudioPathsInner>,
}

impl RobloxStudioPaths {
    /**
        Tries to locate the current Roblox Studio installation and directories.

        # Errors

        - If Roblox Studio is not installed.
        - If Roblox Studio is not supported on the current platform.
        - If the user documents directory (macOS) or local data directory (Windows) cannot be found.
    */
    pub fn new() -> RobloxStudioResult<Self> {
        RobloxStudioPathsInner::new().map(Self::from)
    }

    /**
        Returns the path to the Roblox Studio executable.
    */
    #[must_use]
    pub fn exe(&self) -> &Path {
        self.inner.exe.as_path()
    }

    /**
        Returns the path to the Roblox Studio launcher executable,
        if one is available.
    */
    #[must_use]
    pub fn launcher(&self) -> Option<&Path> {
        self.inner.launcher.as_deref()
    }

    /**
        Returns the preferred executable path for the given task.
    */
    #[must_use]
    pub(crate) fn exe_for_task(&self, task: Option<RobloxStudioTask>) -> &Path {
        if cfg!(target_os = "windows")
            && task.is_some_and(RobloxStudioTask::needs_launcher)
            && self.inner.launcher.is_some()
        {
            self.inner
                .launcher
                .as_deref()
                .expect("launcher path should exist")
        } else {
            self.exe()
        }
    }

    /**
        Returns the path to the Roblox Studio content directory.

        This directory contains Roblox bundled assets, in sub-directories such as:

        - `fonts` - bundled font files, typically in OpenType or TrueType format
        - `sounds` - bundled basic sounds, such as the character reset sound
        - `textures` - bundled texture files, typically used for `CoreGui`
    */
    #[must_use]
    pub fn content(&self) -> &Path {
        self.inner.content.as_path()
    }

    /**
        Returns the path to the Roblox Studio **user plugins** directory.

        For the path to built-in plugins, see [`RobloxStudioPaths::built_in_plugins`].

        # Warning

        This directory may or may not exist as it is created on demand,
        either when a user opens it through the Roblox Studio settings,
        or when they install their first plugin.
    */
    #[must_use]
    pub fn user_plugins(&self) -> &Path {
        self.inner.plugins_user.as_path()
    }

    /**
        Returns the path to the Roblox Studio **built-in plugins** directory.

        These plugins are bundled with Roblox Studio itself, so this directory is part of the
        installation, unlike the user plugins directory ([`RobloxStudioPaths::user_plugins`]).
    */
    #[must_use]
    pub fn built_in_plugins(&self) -> &Path {
        self.inner.plugins_builtin.as_path()
    }

    /**
        Returns the path to the current `GlobalSettings_<version>.xml`, the file Roblox Studio
        stores its global settings in, if one is present.

        The `<version>` suffix is a settings-schema version that Roblox increments over time (it has
        been `_4`, `_8`, `_10`, `_13`, ...), so the highest-versioned file is returned rather than
        assuming a fixed number.
    */
    #[must_use]
    pub fn global_settings(&self) -> Option<PathBuf> {
        let dir = self.inner.settings.as_ref()?;

        let mut best: Option<(u32, PathBuf)> = None;
        for entry in std::fs::read_dir(dir).ok()?.flatten() {
            let file_name = entry.file_name();
            let Some(name) = file_name.to_str() else {
                continue;
            };
            let Some(version) = name
                .strip_prefix("GlobalSettings_")
                .and_then(|rest| rest.strip_suffix(".xml"))
                .and_then(|version| version.parse::<u32>().ok())
            else {
                continue;
            };
            if best
                .as_ref()
                .is_none_or(|(best_version, _)| version > *best_version)
            {
                best = Some((version, entry.path()));
            }
        }

        best.map(|(_, path)| path)
    }
}

// Private inner struct to make RobloxStudioPaths cheaper to clone
#[derive(Debug, Clone)]
struct RobloxStudioPathsInner {
    exe: PathBuf,
    launcher: Option<PathBuf>,
    content: PathBuf,
    plugins_user: PathBuf,
    plugins_builtin: PathBuf,
    settings: Option<PathBuf>,
}

impl From<RobloxStudioPathsInner> for RobloxStudioPaths {
    fn from(inner: RobloxStudioPathsInner) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

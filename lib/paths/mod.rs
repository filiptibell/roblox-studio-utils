use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::RobloxStudioResult;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

/**
    References to discovered, validated paths to the current
    Roblox Studio executable, content, and plugins directories.

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

        These plugins are bundled with Roblox Studio itself, and the directory is guaranteed
        to exist unlike the user plugins directory ([`RobloxStudioPaths::user_plugins`]).
    */
    #[must_use]
    pub fn built_in_plugins(&self) -> &Path {
        self.inner.plugins_builtin.as_path()
    }
}

// Private inner struct to make RobloxStudioPaths cheaper to clone
#[derive(Debug, Clone)]
struct RobloxStudioPathsInner {
    exe: PathBuf,
    content: PathBuf,
    plugins_user: PathBuf,
    plugins_builtin: PathBuf,
}

impl From<RobloxStudioPathsInner> for RobloxStudioPaths {
    fn from(inner: RobloxStudioPathsInner) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

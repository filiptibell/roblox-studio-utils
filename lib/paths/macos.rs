use std::{io, path::PathBuf};

use crate::{RobloxStudioError, RobloxStudioResult};

use super::RobloxStudioPathsInner;

impl RobloxStudioPathsInner {
    pub(super) fn new() -> RobloxStudioResult<Self> {
        let document_dir =
            dirs::document_dir().ok_or(RobloxStudioError::UserDocumentsDirMissing)?;

        let root = app_dirs()
            .map(|dir| dir.join("RobloxStudio.app").join("Contents"))
            .find(|root| root.join("MacOS").join("RobloxStudio").exists())
            .ok_or_else(|| {
                RobloxStudioError::Io(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Roblox Studio installation not found",
                ))
            })?;

        Ok(Self {
            exe: root.join("MacOS").join("RobloxStudio"),
            launcher: None,
            content: root.join("Resources").join("content"),
            plugins_user: document_dir.join("Roblox").join("Plugins"),
            plugins_builtin: root.join("Resources").join("BuiltInPlugins"),
            settings: dirs::home_dir().map(|home| home.join("Library").join("Roblox")),
        })
    }
}

/**
    Directories that Roblox Studio may be installed in, in order of preference.

    The official installer uses `~/Applications` when the user can not write to `/Applications`.
*/
fn app_dirs() -> impl Iterator<Item = PathBuf> {
    [
        Some(PathBuf::from("/Applications")),
        dirs::home_dir().map(|home| home.join("Applications")),
    ]
    .into_iter()
    .flatten()
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::{RobloxStudioError, RobloxStudioPaths};

    use super::app_dirs;

    #[test]
    fn new_checks_that_studio_is_installed() {
        let installed = app_dirs().any(|dir| {
            dir.join("RobloxStudio.app/Contents/MacOS/RobloxStudio")
                .exists()
        });
        match RobloxStudioPaths::new() {
            Ok(_) => assert!(installed),
            Err(RobloxStudioError::Io(e)) => {
                assert!(!installed);
                assert_eq!(e.kind(), io::ErrorKind::NotFound);
            }
            Err(e) => panic!("unexpected error: {e}"),
        }
    }
}

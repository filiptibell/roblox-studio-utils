use std::{io, path::PathBuf};

use crate::{RobloxStudioError, RobloxStudioResult};

use super::RobloxStudioPathsInner;

impl RobloxStudioPathsInner {
    pub(super) fn new() -> RobloxStudioResult<Self> {
        let document_dir =
            dirs::document_dir().ok_or(RobloxStudioError::UserDocumentsDirMissing)?;

        let mut root = PathBuf::from("/Applications");
        root.push("RobloxStudio.app");
        root.push("Contents");

        let exe = root.join("MacOS").join("RobloxStudio");
        if !exe.exists() {
            return Err(RobloxStudioError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "Roblox Studio installation not found",
            )));
        }

        Ok(Self {
            exe,
            launcher: None,
            content: root.join("Resources").join("content"),
            plugins_user: document_dir.join("Roblox").join("Plugins"),
            plugins_builtin: root.join("Resources").join("BuiltInPlugins"),
            settings: dirs::home_dir().map(|home| home.join("Library").join("Roblox")),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{io, path::Path};

    use crate::{RobloxStudioError, RobloxStudioPaths};

    #[test]
    fn new_checks_that_studio_is_installed() {
        let installed =
            Path::new("/Applications/RobloxStudio.app/Contents/MacOS/RobloxStudio").exists();
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

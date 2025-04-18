use std::path::PathBuf;

use crate::{RobloxStudioError, RobloxStudioResult};

use super::RobloxStudioPathsInner;

impl RobloxStudioPathsInner {
    pub(super) fn new() -> RobloxStudioResult<Self> {
        let document_dir =
            dirs::document_dir().ok_or(RobloxStudioError::UserDocumentsDirMissing)?;

        let mut root = PathBuf::from("/Applications");
        root.push("RobloxStudio.app");
        root.push("Contents");

        Ok(Self {
            exe: root.join("MacOS").join("RobloxStudio"),
            content: root.join("Resources").join("content"),
            plugins_user: document_dir.join("Roblox").join("Plugins"),
            plugins_builtin: root.join("Resources").join("BuiltInPlugins"),
        })
    }
}

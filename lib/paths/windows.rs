use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::{RobloxStudioError, RobloxStudioResult};

use super::RobloxStudioPathsInner;

use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

impl RobloxStudioPathsInner {
    pub(super) fn new() -> RobloxStudioResult<Self> {
        let key = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(r"Software\Roblox\RobloxStudio")
            .map_err(RobloxStudioError::Io)?;

        let content: String = key
            .get_value("ContentFolder")
            .map_err(RobloxStudioError::Io)?;
        let content = PathBuf::from(content);

        let root = content.parent().ok_or_else(|| {
            RobloxStudioError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "Malformed registry entry: content folder has no parent directory",
            ))
        })?;

        let plugins_user = dirs::data_local_dir()
            .ok_or(RobloxStudioError::LocalDataDirMissing)?
            .join("Roblox")
            .join("Plugins");

        find_paths_direct(root, &content, &plugins_user)
            .or_else(|| find_paths_versioned(root, &plugins_user))
            .ok_or_else(|| {
                RobloxStudioError::Io(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Roblox Studio installation not found",
                ))
            })
    }
}

fn find_paths_direct(
    root: &Path,
    content: &Path,
    plugins_user: &Path,
) -> Option<RobloxStudioPathsInner> {
    let exe = root.join("RobloxStudioBeta.exe");
    if exe.exists() {
        Some(RobloxStudioPathsInner {
            exe,
            content: content.to_path_buf(),
            plugins_user: plugins_user.to_path_buf(),
            plugins_builtin: root.join("BuiltInPlugins"),
        })
    } else {
        None
    }
}

fn find_paths_versioned(root: &Path, plugins_user: &Path) -> Option<RobloxStudioPathsInner> {
    let versions = root.join("Versions");
    if !versions.is_dir() {
        return None;
    }

    for entry in fs::read_dir(&versions).ok()? {
        let entry = entry.ok()?;
        let Ok(etype) = entry.file_type() else {
            continue;
        };
        if etype.is_dir() {
            let dir = entry.path();
            let exe = dir.join("RobloxStudioBeta.exe");
            if exe.exists() {
                return Some(RobloxStudioPathsInner {
                    exe,
                    content: dir.join("content"),
                    plugins_user: plugins_user.to_path_buf(),
                    plugins_builtin: dir.join("BuiltInPlugins"),
                });
            }
        }
    }

    None
}

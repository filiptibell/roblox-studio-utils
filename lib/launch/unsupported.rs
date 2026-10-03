use std::{io, process::Child};

use crate::result::{RobloxStudioError, RobloxStudioResult};

use super::{Launch, RobloxStudioProcess};

pub(super) const SUPPORTS_BACKGROUND: bool = false;

pub(super) fn run(_launch: &Launch) -> RobloxStudioResult<()> {
    Err(RobloxStudioError::UnsupportedPlatform)
}

pub(super) fn spawn(_launch: &Launch) -> RobloxStudioResult<RobloxStudioProcess> {
    Err(RobloxStudioError::UnsupportedPlatform)
}

pub(super) fn quit_child(_child: &mut Child) -> io::Result<()> {
    Err(io::ErrorKind::Unsupported.into())
}

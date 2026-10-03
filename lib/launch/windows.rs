use std::{
    io,
    os::windows::process::CommandExt,
    process::{Child, Command, Stdio},
};

use crate::result::{RobloxStudioError, RobloxStudioResult};

use super::{Launch, RobloxStudioProcess};

// TODO: Support opening in the background, for example on a separate desktop
// through `STARTUPINFOW::lpDesktop`, or with `SW_SHOWNOACTIVATE`
pub(super) const SUPPORTS_BACKGROUND: bool = false;

/*
    Windows process creation flags & job objects:

    https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
    https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects
*/

const DETACHED_PROCESS: u32 = 0x0000_0008;
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;

const WINDOWS_FLAGS_FULL: u32 =
    DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_BREAKAWAY_FROM_JOB;
const WINDOWS_FLAGS_FALLBACK: u32 = DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP;

pub(super) fn run(launch: &Launch) -> RobloxStudioResult<()> {
    if launch.background {
        return Err(RobloxStudioError::BackgroundUnsupported);
    }

    /*
        Break away from short-lived wrappers on Windows and
        avoid tying Studio to the parent's console process group.

        Some Windows parent processes run inside a job object that does not permit
        `CREATE_BREAKAWAY_FROM_JOB`, which causes Studio launch to fail outright with
        `ERROR_ACCESS_DENIED`. Retry without the breakaway flag so opening Studio still
        works even when we cannot fully escape the parent job.
    */
    spawn_detached_with_flags(launch, WINDOWS_FLAGS_FULL).or_else(|error| match error {
        RobloxStudioError::Io(io_error) if io_error.kind() == io::ErrorKind::PermissionDenied => {
            spawn_detached_with_flags(launch, WINDOWS_FLAGS_FALLBACK)
                .or_else(|_| spawn_detached_with_flags(launch, 0))
        }
        other => Err(other),
    })
}

pub(super) fn spawn(launch: &Launch) -> RobloxStudioResult<RobloxStudioProcess> {
    if launch.background {
        return Err(RobloxStudioError::BackgroundUnsupported);
    }

    Ok(RobloxStudioProcess::new(launch.command().spawn()?))
}

#[allow(clippy::zombie_processes)]
fn spawn_detached_with_flags(launch: &Launch, flags: u32) -> RobloxStudioResult<()> {
    let mut cmd = launch.command();

    if flags != 0 {
        cmd.creation_flags(flags);
    }

    cmd.spawn()?;

    Ok(())
}

pub(super) fn quit_child(child: &mut Child) -> io::Result<()> {
    // Without `/F`, taskkill asks the windows of the process to close
    Command::new("taskkill")
        .args(["/PID", &child.id().to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    Ok(())
}

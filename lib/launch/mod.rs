use std::{
    ffi::OsString,
    fmt, io,
    path::PathBuf,
    process::{Child, ExitStatus},
    thread,
    time::{Duration, Instant},
};

use crate::result::RobloxStudioResult;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
use self::macos as platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use self::unsupported as platform;
#[cfg(target_os = "windows")]
use self::windows as platform;

pub(crate) const SUPPORTS_BACKGROUND: bool = platform::SUPPORTS_BACKGROUND;

const POLL_INTERVAL: Duration = Duration::from_millis(50);

/**
    Everything needed to launch Roblox Studio - each platform decides how.
*/
#[derive(Debug, Clone)]
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
pub(crate) struct Launch {
    pub(crate) exe: PathBuf,
    pub(crate) args: Vec<OsString>,
    pub(crate) background: bool,
}

impl Launch {
    pub(crate) fn run(&self) -> RobloxStudioResult<()> {
        platform::run(self)
    }

    pub(crate) fn spawn(&self) -> RobloxStudioResult<RobloxStudioProcess> {
        platform::spawn(self)
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn command(&self) -> std::process::Command {
        use std::process::{Command, Stdio};

        let mut cmd = Command::new(&self.exe);
        cmd.args(&self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        cmd
    }
}

trait ProcessHandle: fmt::Debug + Send + Sync {
    fn id(&self) -> u32;
    fn try_wait(&mut self) -> io::Result<Option<RobloxStudioExit>>;
    fn wait(&mut self) -> io::Result<RobloxStudioExit>;
    fn quit(&mut self) -> io::Result<()>;
    fn kill(&mut self) -> io::Result<()>;
}

impl ProcessHandle for Child {
    fn id(&self) -> u32 {
        Child::id(self)
    }

    fn try_wait(&mut self) -> io::Result<Option<RobloxStudioExit>> {
        Ok(Child::try_wait(self)?.map(RobloxStudioExit::from))
    }

    fn wait(&mut self) -> io::Result<RobloxStudioExit> {
        Ok(Child::wait(self)?.into())
    }

    fn quit(&mut self) -> io::Result<()> {
        // Once waited for, the identifier of a child may be reused by another process
        if Child::try_wait(self)?.is_some() {
            return Ok(());
        }
        platform::quit_child(self)
    }

    fn kill(&mut self) -> io::Result<()> {
        Child::kill(self)
    }
}

/**
    A handle to a running Roblox Studio process.

    Dropping the handle does not stop the process - use `stop`, `kill`, or
    `quit_after_execution` to make sure that Roblox Studio exits.
*/
#[derive(Debug)]
pub struct RobloxStudioProcess {
    handle: Box<dyn ProcessHandle>,
    killed: bool,
}

impl RobloxStudioProcess {
    #[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
    fn new(handle: impl ProcessHandle + 'static) -> Self {
        Self {
            handle: Box::new(handle),
            killed: false,
        }
    }

    fn mark(&self, exit: RobloxStudioExit) -> RobloxStudioExit {
        RobloxStudioExit {
            killed: self.killed,
            ..exit
        }
    }

    /**
        Returns the OS-assigned process identifier of the Roblox Studio process.
    */
    #[must_use]
    pub fn id(&self) -> u32 {
        self.handle.id()
    }

    /**
        Checks if the Roblox Studio process has exited, without blocking.

        # Errors

        - If the state of the process could not be checked.
    */
    pub fn try_wait(&mut self) -> io::Result<Option<RobloxStudioExit>> {
        Ok(self.handle.try_wait()?.map(|exit| self.mark(exit)))
    }

    /**
        Waits for the Roblox Studio process to exit.

        # Errors

        - If the process could not be waited for.
    */
    pub fn wait(&mut self) -> io::Result<RobloxStudioExit> {
        let exit = self.handle.wait()?;
        Ok(self.mark(exit))
    }

    /**
        Waits for the Roblox Studio process to exit, for at most the given timeout.

        Returns `None` if it is still running after the timeout.

        # Errors

        - If the state of the process could not be checked.
    */
    pub fn wait_timeout(&mut self, timeout: Duration) -> io::Result<Option<RobloxStudioExit>> {
        let Some(deadline) = Instant::now().checked_add(timeout) else {
            return self.wait().map(Some);
        };
        loop {
            if let Some(exit) = self.try_wait()? {
                return Ok(Some(exit));
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Ok(None);
            }
            thread::sleep(remaining.min(POLL_INTERVAL));
        }
    }

    /**
        Asks the Roblox Studio process to quit, the same way as the OS does when it shuts down.

        Roblox Studio may postpone this, for example until a running script has finished,
        so this does not guarantee that it exits - see `stop` for that.

        Does nothing if the process has already exited.

        # Errors

        - If the process could not be asked to quit.
    */
    pub fn quit(&mut self) -> io::Result<()> {
        self.handle.quit()
    }

    /**
        Forces the Roblox Studio process to exit right away, which may lose data that it
        is writing, such as its login - prefer `stop`, which asks it to quit first.

        Does nothing if the process has already exited.

        # Errors

        - If the process could not be killed.
    */
    pub fn kill(&mut self) -> io::Result<()> {
        if self.handle.try_wait()?.is_none() {
            self.handle.kill()?;
            self.killed = true;
        }
        Ok(())
    }

    /**
        Stops the Roblox Studio process, and waits for it to exit.

        This first asks Roblox Studio to quit, and kills it only if it has not exited
        within the given grace period. Once this returns `Ok`, the process has exited.

        To guarantee that Roblox Studio exits within a timeout, use `wait_timeout`
        and then `stop`, which together take at most the timeout plus the grace period.

        # Errors

        - If the process could not be asked to quit, killed, or waited for.
    */
    pub fn stop(&mut self, grace: Duration) -> io::Result<RobloxStudioExit> {
        self.quit()?;
        if let Some(exit) = self.wait_timeout(grace)? {
            return Ok(exit);
        }
        self.kill()?;
        self.wait()
    }
}

/**
    Describes how a Roblox Studio process exited.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RobloxStudioExit {
    status: Option<ExitStatus>,
    killed: bool,
}

impl RobloxStudioExit {
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    const UNKNOWN: Self = Self {
        status: None,
        killed: false,
    };

    /**
        Returns the exit status, if it is known.

        It is not known when Roblox Studio was opened in the background on macOS,
        since the system launches it there, and not the current process.
    */
    #[must_use]
    pub const fn status(&self) -> Option<ExitStatus> {
        self.status
    }

    /**
        Returns whether the process exited successfully, if the exit status is known.
    */
    #[must_use]
    pub fn success(&self) -> Option<bool> {
        self.status.map(|status| status.success())
    }

    /**
        Returns `true` if the process was killed through its handle,
        using `kill`, or by `stop` after the grace period.
    */
    #[must_use]
    pub const fn was_killed(&self) -> bool {
        self.killed
    }
}

impl From<ExitStatus> for RobloxStudioExit {
    fn from(status: ExitStatus) -> Self {
        Self {
            status: Some(status),
            killed: false,
        }
    }
}

impl fmt::Display for RobloxStudioExit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.status {
            Some(status) => status.fmt(f),
            None if self.killed => f.write_str("killed"),
            None => f.write_str("unknown exit status"),
        }
    }
}

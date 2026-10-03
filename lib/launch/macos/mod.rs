use std::{io, os::unix::process::CommandExt, process::Child};

use crate::result::RobloxStudioResult;

use super::{Launch, RobloxStudioProcess};

mod kqueue;
mod workspace;

pub(super) const SUPPORTS_BACKGROUND: bool = true;

#[allow(clippy::zombie_processes)]
pub(super) fn run(launch: &Launch) -> RobloxStudioResult<()> {
    if launch.background {
        workspace::open_hidden(launch)?;
        return Ok(());
    }

    /*
        NOTE: Not waiting on the process here is intentional, we
        are only trying to open Roblox Studio, not get its output,
        and we intentionally don't want toolchain managers such as
        Rokit/Aftman/Foreman to kill and clean up this process either
    */
    let mut cmd = launch.command();

    // Move Studio into a separate session so short-lived wrappers
    // and shell signals do not take it down with the parent process
    let detach = || {
        // SAFETY: `setsid` has no preconditions
        if unsafe { libc::setsid() } == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    };
    // SAFETY: The closure only calls the async-signal-safe `setsid`, and does not allocate
    unsafe { cmd.pre_exec(detach) };

    cmd.spawn()?;

    Ok(())
}

pub(super) fn spawn(launch: &Launch) -> RobloxStudioResult<RobloxStudioProcess> {
    if launch.background {
        let pid = workspace::open_hidden(launch)?;
        // Nothing can stop an app that can not be watched, so it must not keep running
        let watcher = kqueue::ExitWatcher::new(pid).inspect_err(|_| {
            signal(pid, libc::SIGKILL).ok();
        })?;
        return Ok(RobloxStudioProcess::new(watcher));
    }

    Ok(RobloxStudioProcess::new(launch.command().spawn()?))
}

pub(super) fn quit_child(child: &mut Child) -> io::Result<()> {
    signal(child.id().cast_signed(), libc::SIGTERM)
}

fn signal(pid: libc::pid_t, signal: libc::c_int) -> io::Result<()> {
    // SAFETY: Sending a signal has no memory safety requirements
    if unsafe { libc::kill(pid, signal) } == -1 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(error);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        process::{Child, Command},
        thread,
        time::{Duration, Instant},
    };

    use super::{RobloxStudioProcess, kqueue::ExitWatcher};

    fn obeys_quit() -> Child {
        Command::new("sleep").arg("60").spawn().unwrap()
    }

    fn ignores_quit() -> Child {
        let child = Command::new("sh")
            .args(["-c", "trap '' TERM; exec sleep 60"])
            .spawn()
            .unwrap();
        // Give the shell time to ignore the signal before it is sent
        thread::sleep(Duration::from_millis(300));
        child
    }

    /// Like a process opened in the background - the child is only kept to clean it up.
    fn watched(child: &Child) -> RobloxStudioProcess {
        RobloxStudioProcess::new(ExitWatcher::new(child.id().cast_signed()).unwrap())
    }

    #[test]
    fn wait_timeout_returns_none_while_running() {
        let mut child = obeys_quit();
        let mut process = watched(&child);
        let start = Instant::now();
        assert!(
            process
                .wait_timeout(Duration::from_millis(200))
                .unwrap()
                .is_none()
        );
        assert!(start.elapsed() >= Duration::from_millis(200));
        child.kill().unwrap();
        assert!(process.wait_timeout(Duration::MAX).unwrap().is_some());
        child.wait().unwrap();
    }

    #[test]
    fn stop_asks_child_to_quit_first() {
        let mut process = RobloxStudioProcess::new(obeys_quit());
        let exit = process.stop(Duration::from_secs(10)).unwrap();
        assert!(!exit.was_killed());
        assert!(exit.status().is_some());
    }

    #[test]
    fn stop_kills_child_that_does_not_quit() {
        let mut process = RobloxStudioProcess::new(ignores_quit());
        let exit = process.stop(Duration::from_millis(500)).unwrap();
        assert!(exit.was_killed());
        assert!(process.try_wait().unwrap().is_some());
    }

    #[test]
    fn stop_asks_watched_process_to_quit_first() {
        let mut child = obeys_quit();
        let exit = watched(&child).stop(Duration::from_secs(10)).unwrap();
        assert!(!exit.was_killed());
        assert!(exit.status().is_none());
        child.wait().unwrap();
    }

    #[test]
    fn stop_kills_watched_process_that_does_not_quit() {
        let mut child = ignores_quit();
        let mut process = watched(&child);
        let exit = process.stop(Duration::from_millis(500)).unwrap();
        assert!(exit.was_killed());
        // Exited processes must never be signaled again
        process.quit().unwrap();
        process.kill().unwrap();
        child.wait().unwrap();
    }
}

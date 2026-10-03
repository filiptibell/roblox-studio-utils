use std::{
    io,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    ptr,
};

use crate::launch::{ProcessHandle, RobloxStudioExit};

/**
    Watches a process that is not a child of the current process, to find out when it exits.

    A kqueue follows the process itself, so unlike its identifier,
    it can never refer to another process once the original exits.
*/
#[derive(Debug)]
pub(super) struct ExitWatcher {
    pid: libc::pid_t,
    kqueue: OwnedFd,
    exited: bool,
}

impl ExitWatcher {
    pub(super) fn new(pid: libc::pid_t) -> io::Result<Self> {
        let ident = usize::try_from(pid)
            .ok()
            .filter(|&ident| ident > 0)
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;

        // SAFETY: `kqueue` has no preconditions
        let fd = unsafe { libc::kqueue() };
        if fd == -1 {
            return Err(io::Error::last_os_error());
        }

        // SAFETY: The descriptor is valid, and owned by nothing else
        let kqueue = unsafe { OwnedFd::from_raw_fd(fd) };

        let change = libc::kevent {
            ident,
            filter: libc::EVFILT_PROC,
            flags: libc::EV_ADD,
            fflags: libc::NOTE_EXIT,
            data: 0,
            udata: ptr::null_mut(),
        };

        // SAFETY: The change list points to one valid event, and the event list is empty
        let result = unsafe {
            libc::kevent(
                kqueue.as_raw_fd(),
                &raw const change,
                1,
                ptr::null_mut(),
                0,
                ptr::null(),
            )
        };

        // The process may exit before it can be watched
        let exited = result == -1;
        if exited {
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error);
            }
        }

        Ok(Self {
            pid,
            kqueue,
            exited,
        })
    }

    /**
        Returns `true` once the process has exited, waiting for it if `block` is set.
    */
    fn poll(&mut self, block: bool) -> io::Result<bool> {
        let zero = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        let timeout = if block { ptr::null() } else { &raw const zero };
        let mut event = libc::kevent {
            ident: 0,
            filter: 0,
            flags: 0,
            fflags: 0,
            data: 0,
            udata: ptr::null_mut(),
        };
        while !self.exited {
            // SAFETY: The change list is empty, the event list has room for one
            // event, and the timeout is either null or points to a valid timespec
            let result = unsafe {
                libc::kevent(
                    self.kqueue.as_raw_fd(),
                    ptr::null(),
                    0,
                    &raw mut event,
                    1,
                    timeout,
                )
            };
            match result {
                -1 if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted => {}
                -1 => return Err(io::Error::last_os_error()),
                0 => return Ok(false),
                _ => self.exited = true,
            }
        }
        Ok(true)
    }
}

impl ProcessHandle for ExitWatcher {
    fn id(&self) -> u32 {
        self.pid.cast_unsigned()
    }

    fn try_wait(&mut self) -> io::Result<Option<RobloxStudioExit>> {
        Ok(self.poll(false)?.then_some(RobloxStudioExit::UNKNOWN))
    }

    fn wait(&mut self) -> io::Result<RobloxStudioExit> {
        self.poll(true)?;
        Ok(RobloxStudioExit::UNKNOWN)
    }

    fn quit(&mut self) -> io::Result<()> {
        self.signal(libc::SIGTERM)
    }

    fn kill(&mut self) -> io::Result<()> {
        self.signal(libc::SIGKILL)
    }
}

impl ExitWatcher {
    /**
        Sends a signal to the process, unless it has exited - its identifier may then
        belong to another process. Only the tiny window between the check and the
        signal remains, since macOS has no way to signal a process by anything else.
    */
    fn signal(&mut self, signal: libc::c_int) -> io::Result<()> {
        if self.poll(false)? {
            return Ok(());
        }
        super::signal(self.pid, signal)
    }
}

#[cfg(test)]
mod tests {
    use std::process::{Child, Command};

    use super::{ExitWatcher, ProcessHandle};

    fn watch(child: &Child) -> ExitWatcher {
        ExitWatcher::new(child.id().cast_signed()).unwrap()
    }

    #[test]
    fn running_process_has_not_exited() {
        let mut child = Command::new("sleep").arg("60").spawn().unwrap();
        assert!(watch(&child).try_wait().unwrap().is_none());
        child.kill().unwrap();
        child.wait().unwrap();
    }

    #[test]
    fn killed_process_has_exited() {
        let mut child = Command::new("sleep").arg("60").spawn().unwrap();
        let mut watcher = watch(&child);
        watcher.kill().unwrap();
        watcher.wait().unwrap();
        assert!(watcher.try_wait().unwrap().is_some());
        watcher.kill().unwrap();
        child.wait().unwrap();
    }

    #[test]
    fn process_that_exited_before_watching_has_exited() {
        let mut child = Command::new("true").spawn().unwrap();
        let pid = child.id().cast_signed();
        child.wait().unwrap();
        let mut watcher = ExitWatcher::new(pid).unwrap();
        assert!(watcher.try_wait().unwrap().is_some());
    }

    #[test]
    fn invalid_process_identifier_is_rejected() {
        assert!(ExitWatcher::new(0).is_err());
        assert!(ExitWatcher::new(-1).is_err());
    }
}

use std::{
    ffi::OsString,
    fs, io,
    net::Ipv4Addr,
    path::{self, Path},
    process::{Child, Command, Stdio},
};

use crate::paths::RobloxStudioPaths;
use crate::result::{RobloxStudioError, RobloxStudioResult};
use crate::task::RobloxStudioTask;

const DEFAULT_SERVER_ADDR: Ipv4Addr = Ipv4Addr::LOCALHOST;
const DEFAULT_SERVER_PORT: u16 = 50608;

/**
    A builder to open a Roblox Studio instance through the official binary,
    while also properly handling its CLI arguments and intricacies.
*/
#[derive(Debug, Clone)]
pub struct RobloxStudioOpener {
    args: Vec<OsString>,
    task: Option<RobloxStudioTask>,
    server_addr: Ipv4Addr,
    server_port: u16,
}

impl RobloxStudioOpener {
    /**
        Create a new Roblox Studio opener.
    */
    #[must_use]
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            task: None,
            server_addr: DEFAULT_SERVER_ADDR,
            server_port: DEFAULT_SERVER_PORT,
        }
    }

    /**
        Add a key-value argument pair to the Roblox Studio opener.

        This should typically not be used - try to use the more specific
        methods such as `edit_place` or `edit_file` instead when possible.
    */
    #[must_use]
    #[doc(hidden)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn with_arg<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<OsString>,
        V: Into<OsString>,
    {
        let key: OsString = key.into();
        let value: OsString = value.into();
        if key == "-task"
            && let Some(task_str) = value.to_str()
        {
            self.task = RobloxStudioTask::parse(task_str);
        }
        self.args.push(key);
        self.args.push(value);
        self
    }

    /**
        Add a flag argument, which has no value, to the Roblox Studio opener.
    */
    fn with_flag<K>(mut self, key: K) -> Self
    where
        K: Into<OsString>,
    {
        self.args.push(key.into());
        self
    }

    /**
        Adds creator, universe, and place id arguments, filled with zeros.
    */
    fn with_zeros(self) -> Self {
        // Necessary for some commands even though they are
        // unused - maybe these can be removed in the future?
        self.with_arg("-creatorType", "0")
            .with_arg("-creatorId", "0")
            .with_arg("-universeId", "0")
            .with_arg("-placeId", "0")
    }

    /**
        Sets a custom server address to use with the `start_server`,
        `start_server_with_place`, or `start_client` methods.

        Defaults to localhost (`127.0.0.1`).
    */
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn with_server_addr<A>(mut self, server_addr: A) -> Self
    where
        A: Into<Ipv4Addr>,
    {
        self.server_addr = server_addr.into();
        self
    }

    /**
        Sets a custom server port to use with the `start_server`,
        `start_server_with_place`, or `start_client` methods.

        Defaults to port `50608`.
    */
    #[must_use]
    pub fn with_server_port(mut self, server_port: u16) -> Self {
        self.server_port = server_port;
        self
    }

    /**
        Edit an online place in Roblox Studio.

        This will open the place with the given `universe_id` and `place_id`.
    */
    #[must_use]
    pub fn open_place(mut self, universe_id: u64, place_id: u64) -> Self {
        self.task = Some(RobloxStudioTask::EditPlace);
        self.with_arg("-task", RobloxStudioTask::EditPlace)
            .with_arg("-universeId", universe_id.to_string())
            .with_arg("-placeId", place_id.to_string())
    }

    /**
        Edit a local place file in Roblox Studio.

        This will open the place file at the given `file_path`.

        # Errors

        - If the given `file_path` cannot be canonicalized.
        - If the given `file_path` cannot be converted to a string.
    */
    pub fn open_file<P>(mut self, file_path: P) -> RobloxStudioResult<Self>
    where
        P: AsRef<Path>,
    {
        self.task = Some(RobloxStudioTask::EditFile);
        let file_path_str = canonicalize_to_string(file_path.as_ref())?;
        Ok(self
            .with_arg("-task", RobloxStudioTask::EditFile)
            .with_arg("-localPlaceFile", file_path_str))
    }

    /**
        Start a server in Roblox Studio with the given place file.

        This will copy the place file at the given `file_path`
        to the Roblox server file, and then start the server.

        # Errors

        - If the local data directory cannot be found.
        - If the given place file cannot be copied to the local data directory.
    */
    pub fn start_server<P>(mut self, file_path: P) -> RobloxStudioResult<Self>
    where
        P: AsRef<Path>,
    {
        self.task = Some(RobloxStudioTask::StartServer);
        let file_path_source = file_path
            .as_ref()
            .canonicalize()
            .map_err(|e| RobloxStudioError::PathCanonicalize(e.to_string()))?;
        let file_path_target = dirs::data_local_dir()
            .ok_or(RobloxStudioError::LocalDataDirMissing)?
            .join("Roblox")
            .join("server.rbxl");

        fs::copy(file_path_source, file_path_target)
            .map_err(|e| RobloxStudioError::LocalDataDirCopyPlace(e.to_string()))?;

        let server_addr = self.server_addr.to_string();
        let server_port = self.server_port.to_string();
        Ok(self
            .with_arg("-task", RobloxStudioTask::StartServer)
            .with_arg("-server", server_addr)
            .with_arg("-port", server_port)
            .with_zeros())
    }

    /**
        Start a server in Roblox Studio with the given place file and clients.

        This will also automatically start the given number of clients.

        See `start_server` for more information.
    */
    #[allow(clippy::missing_errors_doc)]
    pub fn start_server_with_clients<P>(
        self,
        file_path: P,
        num_clients: u8,
    ) -> RobloxStudioResult<Self>
    where
        P: AsRef<Path>,
    {
        Ok(self
            .start_server(file_path)?
            .with_arg("-numtestserverplayersuponstartup", num_clients.to_string()))
    }

    /**
        Starts a single client, connecting to an already launched server.

        See `start_server` for more information.
    */
    #[must_use]
    pub fn start_client(mut self) -> Self {
        self.task = Some(RobloxStudioTask::StartClient);
        let server_addr = self.server_addr.to_string();
        let server_port = self.server_port.to_string();
        self.with_arg("-task", RobloxStudioTask::StartClient)
            .with_arg("-server", server_addr)
            .with_arg("-port", server_port)
            .with_zeros()
    }

    /**
        Run a Luau script in the default baseplate template in Roblox Studio.

        The script runs after the place has loaded, at the same
        permission level as the Roblox Studio command bar.

        # Errors

        - If the given `script_path` cannot be canonicalized.
        - If the given `script_path` cannot be converted to a string.
    */
    pub fn run_script<S>(mut self, script_path: S) -> RobloxStudioResult<Self>
    where
        S: AsRef<Path>,
    {
        self.task = Some(RobloxStudioTask::RunScript);
        let script_path_str = canonicalize_to_string(script_path.as_ref())?;
        Ok(self
            .with_arg("-task", RobloxStudioTask::RunScript)
            .with_arg("-runScriptFile", script_path_str))
    }

    /**
        Run a Luau script in an online place in Roblox Studio.

        This will open the place with the given `universe_id` and `place_id`,
        and run the script after the place has loaded.

        See `run_script` for more information.

        # Errors

        - If the given `script_path` cannot be canonicalized.
        - If the given `script_path` cannot be converted to a string.
    */
    pub fn run_script_in_place<S>(
        self,
        universe_id: u64,
        place_id: u64,
        script_path: S,
    ) -> RobloxStudioResult<Self>
    where
        S: AsRef<Path>,
    {
        Ok(self
            .run_script(script_path)?
            .with_arg("-universeId", universe_id.to_string())
            .with_arg("-placeId", place_id.to_string()))
    }

    /**
        Run a Luau script in a local place file in Roblox Studio.

        This will open the place file at the given `file_path`,
        and run the script after the place has loaded.

        See `run_script` for more information.

        # Errors

        - If the given `file_path` or `script_path` cannot be canonicalized.
        - If the given `file_path` or `script_path` cannot be converted to a string.
    */
    pub fn run_script_in_file<P, S>(self, file_path: P, script_path: S) -> RobloxStudioResult<Self>
    where
        P: AsRef<Path>,
        S: AsRef<Path>,
    {
        let file_path_str = canonicalize_to_string(file_path.as_ref())?;
        Ok(self
            .run_script(script_path)?
            .with_arg("-localPlaceFile", file_path_str))
    }

    /**
        Sets a file that the output of a script is written to, when used
        together with `run_script`, `run_script_in_place`, or `run_script_in_file`.

        The file does not need to exist. Roblox Studio writes to the file when
        it closes, so this is mostly useful together with `quit_after_execution`.

        # Errors

        - If the given `output_path` cannot be made absolute.
        - If the given `output_path` cannot be converted to a string.
    */
    pub fn with_output_file<P>(self, output_path: P) -> RobloxStudioResult<Self>
    where
        P: AsRef<Path>,
    {
        let output_path_full = path::absolute(output_path.as_ref())?;
        let output_path_str = output_path_full
            .to_str()
            .ok_or_else(|| RobloxStudioError::PathToString(output_path_full.clone()))?
            .to_string();
        Ok(self.with_arg("-outputFile", output_path_str))
    }

    /**
        Makes Roblox Studio close after a script has finished running, when used
        together with `run_script`, `run_script_in_place`, or `run_script_in_file`.

        Note that Roblox Studio exits successfully even if the script raises an error.
        Use `with_output_file` to read the output of the script, including any errors.
    */
    #[must_use]
    pub fn quit_after_execution(self) -> Self {
        self.with_flag("-quitAfterExecution")
    }

    /**
        Starts Roblox Studio with all of the given arguments.

        Note that this will not wait for Roblox Studio to actually
        open the file/server/client - it only guarantees that the process
        has been spawned and that it has received the necessary arguments.

        # Errors

        - If the Roblox Studio executable cannot be found.
    */
    #[allow(clippy::zombie_processes)]
    pub fn run(self) -> RobloxStudioResult<()> {
        let paths = RobloxStudioPaths::new()?;
        let exe = paths.exe_for_task(self.task);

        spawn_studio_process(exe, &self.args)?;

        Ok(())
    }

    /**
        Starts Roblox Studio with all of the given arguments,
        and returns a handle to the Roblox Studio process.

        Unlike `run`, the process is not detached from the current process.
        The caller can wait for it to exit, or kill it, using the returned
        handle. This is mostly useful together with `quit_after_execution`.

        Note that on Windows, tasks that need the Roblox Studio launcher, such as
        `EditPlace`, return a handle to the launcher process instead of Roblox Studio.

        # Errors

        - If the Roblox Studio executable cannot be found.
    */
    pub fn spawn(self) -> RobloxStudioResult<Child> {
        let paths = RobloxStudioPaths::new()?;
        let exe = paths.exe_for_task(self.task);

        let child = create_studio_command(exe, &self.args).spawn()?;

        Ok(child)
    }
}

impl Default for RobloxStudioOpener {
    fn default() -> Self {
        Self::new()
    }
}

fn canonicalize_to_string(path: &Path) -> RobloxStudioResult<String> {
    let path_full = path
        .canonicalize()
        .map_err(|e| RobloxStudioError::PathCanonicalize(e.to_string()))?;
    let path_str = path_full
        .to_str()
        .ok_or_else(|| RobloxStudioError::PathToString(path_full.clone()))?;
    Ok(path_str.to_string())
}

fn create_studio_command(exe: &Path, args: &[OsString]) -> Command {
    let mut cmd = Command::new(exe);
    cmd.args(args);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    cmd
}

#[cfg(not(target_os = "windows"))]
#[allow(clippy::zombie_processes)]
fn spawn_studio_process(exe: &Path, args: &[OsString]) -> RobloxStudioResult<()> {
    let mut cmd = create_studio_command(exe, args);

    /*
        NOTE: Not waiting on the process here is intentional, we
        are only trying to open Roblox Studio, not get its output,
        and we intentionally don't want toolchain managers such as
        Rokit/Aftman/Foreman to kill and clean up this process either
    */
    configure_detached_spawn(&mut cmd)?;

    cmd.spawn()?;

    Ok(())
}

#[cfg(target_os = "windows")]
#[allow(clippy::zombie_processes)]
fn spawn_studio_process(exe: &Path, args: &[OsString]) -> RobloxStudioResult<()> {
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

    /*
        Break away from short-lived wrappers on Windows and
        avoid tying Studio to the parent's console process group.

        Some Windows parent processes run inside a job object that does not permit
        `CREATE_BREAKAWAY_FROM_JOB`, which causes Studio launch to fail outright with
        `ERROR_ACCESS_DENIED`. Retry without the breakaway flag so opening Studio still
        works even when we cannot fully escape the parent job.
    */
    spawn_studio_process_with_flags(exe, args, WINDOWS_FLAGS_FULL).or_else(|error| match error {
        RobloxStudioError::Io(io_error) if io_error.kind() == io::ErrorKind::PermissionDenied => {
            spawn_studio_process_with_flags(exe, args, WINDOWS_FLAGS_FALLBACK)
                .or_else(|_| spawn_studio_process_with_flags(exe, args, 0))
        }
        other => Err(other),
    })
}

#[cfg(target_os = "windows")]
#[allow(clippy::zombie_processes)]
fn spawn_studio_process_with_flags(
    exe: &Path,
    args: &[OsString],
    flags: u32,
) -> RobloxStudioResult<()> {
    let mut cmd = create_studio_command(exe, args);

    configure_detached_spawn(&mut cmd, flags)?;

    cmd.spawn()?;

    Ok(())
}

#[cfg(target_family = "unix")]
fn configure_detached_spawn(cmd: &mut Command) -> io::Result<()> {
    use std::os::unix::process::CommandExt;

    /*
        Move Studio into a separate session so short-lived wrappers
        and shell signals do not take it down with the parent process.

        SAFETY: The closure only calls async-signal-safe `setsid` and returns
        an OS error directly, which is the intended `pre_exec` usage.
    */
    unsafe {
        cmd.pre_exec(|| {
            if libc::setsid() == -1 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn configure_detached_spawn(cmd: &mut Command, flags: u32) -> io::Result<()> {
    use std::os::windows::process::CommandExt;

    if flags != 0 {
        cmd.creation_flags(flags);
    }

    Ok(())
}

#[cfg(not(any(target_family = "unix", target_os = "windows")))]
fn configure_detached_spawn(_cmd: &mut Command) -> io::Result<()> {
    Ok(())
}

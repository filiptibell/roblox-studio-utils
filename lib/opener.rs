use std::{
    ffi::OsString,
    fs,
    net::Ipv4Addr,
    path::Path,
    process::{Command, Stdio},
};

use crate::paths::RobloxStudioPaths;
use crate::result::{RobloxStudioError, RobloxStudioResult};
use crate::task::RobloxStudioTask;

const DEFAULT_SERVER_ADDR: Ipv4Addr = Ipv4Addr::LOCALHOST;
const DEFAULT_SERVER_PORT: u16 = 50608;

/**
    A wrapper around the `opener` crate to open Roblox Studio natively,
    while also properly handling its CLI arguments and intricacies.
*/
#[derive(Debug, Clone)]
pub struct RobloxStudioOpener {
    args: Vec<OsString>,
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
        self.args.push(key.into());
        self.args.push(value.into());
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
    pub fn open_place(self, universe_id: u64, place_id: u64) -> Self {
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
    pub fn open_file<P>(self, file_path: P) -> RobloxStudioResult<Self>
    where
        P: AsRef<Path>,
    {
        let file_path_full = file_path
            .as_ref()
            .canonicalize()
            .map_err(|e| RobloxStudioError::PathCanonicalize(e.to_string()))?;
        let file_path_str = file_path_full
            .to_str()
            .ok_or(RobloxStudioError::PathToString(file_path_full.clone()))?;
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
    pub fn start_server<P>(self, file_path: P) -> RobloxStudioResult<Self>
    where
        P: AsRef<Path>,
    {
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
    pub fn start_client(self) -> Self {
        let server_addr = self.server_addr.to_string();
        let server_port = self.server_port.to_string();
        self.with_arg("-task", RobloxStudioTask::StartClient)
            .with_arg("-server", server_addr)
            .with_arg("-port", server_port)
            .with_zeros()
    }

    /**
        Starts Roblox Studio with all of the given arguments.

        Note that this will not wait for Roblox Studio to actually
        open the file/server/client - it only guarantees that the process
        has been spawned and that it has received the necessary arguments.

        # Errors

        - If the Roblox Studio executable cannot be found.
    */
    pub fn run(self) -> RobloxStudioResult<()> {
        let paths = RobloxStudioPaths::new()?;

        let mut cmd = Command::new(paths.exe());
        cmd.args(self.args);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        // NOTE: Not waiting on the process here is intentional, we
        // are only trying to open Roblox Studio, not get its output,
        // and we intentionally don't want toolchain managers such as
        // Rokit/Aftman/Foreman to kill and clean up this process either
        #[allow(clippy::zombie_processes)]
        cmd.spawn()?;

        Ok(())
    }
}

impl Default for RobloxStudioOpener {
    fn default() -> Self {
        Self::new()
    }
}

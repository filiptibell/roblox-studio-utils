use std::{net::Ipv4Addr, path::Path};

use crate::result::{RobloxStudioError, RobloxStudioResult};
use crate::task::RobloxStudioTask;

// FUTURE: Maybe these can be made configurable, too?

const SERVER_ADDR: Ipv4Addr = Ipv4Addr::LOCALHOST;
const SERVER_PORT: u16 = 50608;

/**
    A wrapper around the `opener` crate to open Roblox Studio natively,
    while also properly handling its CLI arguments and intricacies.
*/
#[derive(Debug, Clone)]
pub struct RobloxStudioOpener {
    args: Vec<(String, String)>,
}

impl RobloxStudioOpener {
    /**
        Create a new Roblox Studio opener.
    */
    #[must_use]
    pub fn new() -> Self {
        Self {
            args: vec![(String::from("roblox-studio"), String::from("1"))],
        }
    }

    /**
        Add an argument to the Roblox Studio opener.

        This should typically not be used - try to use the more specific
        methods such as `edit_place` or `edit_file` instead when possible.
    */
    #[must_use]
    #[doc(hidden)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn with_arg<K, V>(mut self, key: K, value: V) -> Self
    where
        K: ToString,
        V: ToString,
    {
        self.args.push((key.to_string(), value.to_string()));
        self
    }

    /**
        Adds creator, universe, and place id arguments, filled with zeros.
    */
    fn with_zeros(self) -> Self {
        // Necessary for some commands even though they are
        // unused - maybe these can be removed in the future?
        self.with_arg("creatorType", "0")
            .with_arg("creatorId", "0")
            .with_arg("universeId", "0")
            .with_arg("placeId", "0")
    }

    /**
        Edit an online place in Roblox Studio.

        This will open the place with the given `universe_id` and `place_id`.
    */
    #[must_use]
    pub fn open_place(self, universe_id: u64, place_id: u64) -> Self {
        self.with_arg("task", RobloxStudioTask::EditPlace)
            .with_arg("universeId", universe_id)
            .with_arg("placeId", place_id)
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
            .with_arg("task", RobloxStudioTask::EditFile)
            .with_arg("localPlaceFile", file_path_str))
    }

    /**
        Start a server in Roblox Studio with the given place file.

        This will copy the place file at the given `file_path`
        to the Roblox server file, and then start the server.

        NOTE: This is a blocking operation.

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

        std::fs::copy(file_path_source, file_path_target)
            .map_err(|e| RobloxStudioError::LocalDataDirCopyPlace(e.to_string()))?;

        Ok(self
            .with_arg("task", RobloxStudioTask::StartServer)
            .with_arg("-server", SERVER_ADDR)
            .with_arg("-port", SERVER_PORT)
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
            .with_arg("numtestserverplayersuponstartup", num_clients))
    }

    /**
        Starts a single client, connecting to an already launched server.

        See `start_server` for more information.
    */
    #[must_use]
    pub fn start_client(self) -> Self {
        self.with_arg("task", RobloxStudioTask::StartClient)
            .with_arg("-server", SERVER_ADDR)
            .with_arg("-port", SERVER_PORT)
            .with_zeros()
    }

    /**
        Starts Roblox Studio with all of the given arguments.

        NOTE: This is a blocking operation.

        # Errors

        - If Roblox Studio cannot be opened.
    */
    pub fn run(self) -> RobloxStudioResult<()> {
        let args = build_args_string(self.args);
        opener::open(args)?;
        Ok(())
    }
}

impl Default for RobloxStudioOpener {
    fn default() -> Self {
        Self::new()
    }
}

fn build_args_string(args: Vec<(String, String)>) -> String {
    args.into_iter()
        .map(|(k, v)| format!("{k}:{v}"))
        .collect::<Vec<_>>()
        .join("+")
}

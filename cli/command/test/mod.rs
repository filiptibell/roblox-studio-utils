use clap::{Args, Subcommand};

use crate::common::{CliResult, Context};

pub mod client;
pub mod server;
pub mod session;

#[derive(Debug, Args)]
pub struct TestCommand {
    #[command(subcommand)]
    command: TestSubcommand,
}

#[derive(Debug, Subcommand)]
enum TestSubcommand {
    /// Start a local Roblox Studio test server with a place file.
    Server(server::ServerCommand),
    /// Start a Roblox Studio test client connected to an existing server.
    Client(client::ClientCommand),
    /// Start a local test server and launch clients against it.
    Session(session::SessionCommand),
}

impl TestCommand {
    pub fn run(self, context: Context) -> CliResult {
        match self.command {
            TestSubcommand::Server(command) => command.run(context),
            TestSubcommand::Client(command) => command.run(context),
            TestSubcommand::Session(command) => command.run(context),
        }
    }
}

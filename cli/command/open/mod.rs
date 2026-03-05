use clap::{Args, Subcommand};

use crate::common::{CliResult, Context};

pub mod file;
pub mod place;

#[derive(Debug, Args)]
pub struct OpenCommand {
    #[command(subcommand)]
    command: OpenSubcommand,
}

#[derive(Debug, Subcommand)]
enum OpenSubcommand {
    /// Open an online place in Roblox Studio.
    Place(place::PlaceCommand),
    /// Open a local place file in Roblox Studio.
    File(file::FileCommand),
}

impl OpenCommand {
    pub fn run(self, context: Context) -> CliResult {
        match self.command {
            OpenSubcommand::Place(command) => command.run(context),
            OpenSubcommand::File(command) => command.run(context),
        }
    }
}

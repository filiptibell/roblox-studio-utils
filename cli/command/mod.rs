use clap::Subcommand;

use crate::common::{CliResult, Context};

pub mod doctor;
pub mod open;
pub mod paths;
pub mod run;
pub mod test;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Print the detected Roblox Studio installation paths.
    Paths(paths::PathsCommand),
    /// Inspect the local Roblox Studio setup and launch behavior.
    Doctor(doctor::DoctorCommand),
    /// Open a place in Roblox Studio.
    Open(open::OpenCommand),
    /// Run a Luau script in Roblox Studio.
    Run(run::RunCommand),
    /// Launch local Roblox Studio testing workflows.
    Test(test::TestCommand),
}

impl Command {
    pub fn run(self, context: Context) -> CliResult {
        match self {
            Self::Paths(command) => command.run(),
            Self::Doctor(command) => command.run(),
            Self::Open(command) => command.run(context),
            Self::Run(command) => command.run(context),
            Self::Test(command) => command.run(context),
        }
    }
}

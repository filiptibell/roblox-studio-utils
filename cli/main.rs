use clap::Parser;

use crate::command::Command;
use crate::common::{CliResult, Context};

mod command;
mod common;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Command-line utility for interacting with Roblox Studio"
)]
struct Cli {
    /// Suppress launch confirmation output.
    #[arg(short, long, global = true)]
    quiet: bool,
    #[command(subcommand)]
    command: Command,
}

fn main() -> CliResult {
    let cli = Cli::parse();
    let context = Context::new(cli.quiet);
    cli.command.run(context)
}

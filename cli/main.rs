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
    /// Open Roblox Studio in the background, without activating it or showing its windows.
    #[arg(short, long, global = true)]
    background: bool,
    #[command(subcommand)]
    command: Command,
}

fn main() -> CliResult {
    let cli = Cli::parse();
    let context = Context::new(cli.quiet, cli.background);
    cli.command.run(context)
}

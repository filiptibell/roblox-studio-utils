use std::path::PathBuf;

use anyhow::bail;
use clap::Args;

use roblox_studio_utils::RobloxStudioOpener;

use crate::common::{CliResult, Context};

#[derive(Debug, Args)]
pub struct RunCommand {
    script_path: PathBuf,
    /// The universe that owns the online place to run the script in.
    #[arg(long, requires = "place_id", conflicts_with = "file")]
    universe_id: Option<u64>,
    /// The online place to run the script in.
    #[arg(long, requires = "universe_id", conflicts_with = "file")]
    place_id: Option<u64>,
    /// The local place file to run the script in.
    #[arg(long)]
    file: Option<PathBuf>,
    /// Write the output of the script to this file, when Roblox Studio closes.
    #[arg(long)]
    output_file: Option<PathBuf>,
    /// Close Roblox Studio after the script has finished, and wait for it to close.
    #[arg(long)]
    quit_after_execution: bool,
}

impl RunCommand {
    pub fn run(self, context: Context) -> CliResult {
        let opener = RobloxStudioOpener::new();
        let mut opener = match (self.universe_id.zip(self.place_id), &self.file) {
            (Some((universe_id, place_id)), _) => {
                opener.run_script_in_place(universe_id, place_id, &self.script_path)?
            }
            (None, Some(file_path)) => opener.run_script_in_file(file_path, &self.script_path)?,
            (None, None) => opener.run_script(&self.script_path)?,
        };

        if let Some(output_file) = &self.output_file {
            opener = opener.with_output_file(output_file)?;
        }

        let script_path = self.script_path.display();
        if self.quit_after_execution {
            let status = opener.quit_after_execution().spawn()?.wait()?;
            if !status.success() {
                bail!("Roblox Studio exited with {status}");
            }
            context.print(format!("Roblox Studio finished running {script_path}."));
        } else {
            opener.run()?;
            context.print(format!("Launched Roblox Studio to run {script_path}."));
        }

        Ok(())
    }
}

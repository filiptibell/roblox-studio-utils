use std::path::PathBuf;

use clap::Args;

use roblox_studio_utils::RobloxStudioOpener;

use crate::common::{CliResult, Context};

#[derive(Debug, Args)]
pub struct FileCommand {
    file_path: PathBuf,
}

impl FileCommand {
    pub fn run(self, context: Context) -> CliResult {
        RobloxStudioOpener::new()
            .open_file(&self.file_path)?
            .run()?;

        let file_path = self.file_path.display();
        context.print(format!("Launched Roblox Studio for {file_path}."));

        Ok(())
    }
}

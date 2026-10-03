use std::path::PathBuf;

use clap::Args;

use crate::common::{CliResult, Context};

#[derive(Debug, Args)]
pub struct FileCommand {
    file_path: PathBuf,
}

impl FileCommand {
    pub fn run(self, context: Context) -> CliResult {
        context.opener().open_file(&self.file_path)?.run()?;

        let file_path = self.file_path.display();
        context.print(format!("Launched Roblox Studio for {file_path}."));

        Ok(())
    }
}

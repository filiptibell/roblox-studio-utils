use std::path::PathBuf;

use clap::Args;

use crate::common::{CliResult, Context, ServerOptions};

#[derive(Debug, Args)]
pub struct ServerCommand {
    file_path: PathBuf,
    #[command(flatten)]
    server: ServerOptions,
}

impl ServerCommand {
    pub fn run(self, context: Context) -> CliResult {
        let endpoint = self.server.endpoint();
        self.server
            .apply(context.opener())
            .start_server(&self.file_path)?
            .run()?;

        let file_path = self.file_path.display();
        context.print(format!(
            "Launched Roblox Studio test server for {file_path} on {endpoint}."
        ));

        Ok(())
    }
}

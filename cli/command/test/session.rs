use std::path::PathBuf;

use clap::Args;

use roblox_studio_utils::RobloxStudioOpener;

use crate::common::{CliResult, Context, ServerOptions};

#[derive(Debug, Args)]
pub struct SessionCommand {
    file_path: PathBuf,
    #[arg(long)]
    clients: u8,
    #[command(flatten)]
    server: ServerOptions,
}

impl SessionCommand {
    pub fn run(self, context: Context) -> CliResult {
        let endpoint = self.server.endpoint();
        self.server
            .apply(RobloxStudioOpener::new())
            .start_server_with_clients(&self.file_path, self.clients)?
            .run()?;

        let clients = self.clients;
        let file_path = self.file_path.display();
        context.print(format!(
            "Launched Roblox Studio test session for {file_path} with {clients} clients on {endpoint}."
        ));

        Ok(())
    }
}

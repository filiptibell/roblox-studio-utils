use clap::Args;

use crate::common::{CliResult, Context, ServerOptions};

#[derive(Debug, Args)]
pub struct ClientCommand {
    #[command(flatten)]
    server: ServerOptions,
}

impl ClientCommand {
    pub fn run(self, context: Context) -> CliResult {
        let endpoint = self.server.endpoint();
        self.server.apply(context.opener()).start_client().run()?;

        context.print(format!(
            "Launched Roblox Studio test client for {endpoint}."
        ));

        Ok(())
    }
}

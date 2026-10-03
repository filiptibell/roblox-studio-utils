use clap::Args;

use crate::common::{CliResult, Context};

#[derive(Debug, Args)]
pub struct PlaceCommand {
    universe_id: u64,
    place_id: u64,
}

impl PlaceCommand {
    pub fn run(self, context: Context) -> CliResult {
        context
            .opener()
            .open_place(self.universe_id, self.place_id)
            .run()?;

        let universe_id = self.universe_id;
        let place_id = self.place_id;
        context.print(format!(
            "Launched Roblox Studio for universe {universe_id} place {place_id}."
        ));

        Ok(())
    }
}

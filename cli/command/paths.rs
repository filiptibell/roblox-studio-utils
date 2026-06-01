use clap::Args;
use serde::Serialize;

use roblox_studio_utils::RobloxStudioPaths;

use crate::common::CliResult;

#[derive(Debug, Args)]
pub struct PathsCommand {
    /// Print the output as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PathsReport {
    executable: String,
    content: String,
    built_in_plugins: String,
    user_plugins: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    global_settings: Option<String>,
}

impl PathsCommand {
    pub fn run(self) -> CliResult {
        let paths = RobloxStudioPaths::new()?;
        let report = PathsReport {
            executable: paths.exe().display().to_string(),
            content: paths.content().display().to_string(),
            built_in_plugins: paths.built_in_plugins().display().to_string(),
            user_plugins: paths.user_plugins().display().to_string(),
            global_settings: paths
                .global_settings()
                .map(|path| path.display().to_string()),
        };

        if self.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!("Paths to current Roblox Studio installation:");
            println!();
            println!("- Executable:       {}", report.executable);
            println!("- Content:          {}", report.content);
            println!("- Built-in Plugins: {}", report.built_in_plugins);
            println!("- User Plugins:     {}", report.user_plugins);
            println!(
                "- Global Settings:  {}",
                report.global_settings.as_deref().unwrap_or("(none found)")
            );
        }

        Ok(())
    }
}

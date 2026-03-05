use std::{env, path::Path};

use clap::Args;
use serde::Serialize;

use roblox_studio_utils::RobloxStudioPaths;

use crate::common::CliResult;

#[derive(Debug, Args)]
pub struct DoctorCommand {
    /// Print the output as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DoctorReport {
    ok: bool,
    os: &'static str,
    arch: &'static str,
    executable: Option<PathStatus>,
    content: Option<PathStatus>,
    built_in_plugins: Option<PathStatus>,
    user_plugins: Option<PathStatus>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct PathStatus {
    path: String,
    exists: bool,
}

impl DoctorCommand {
    pub fn run(self) -> CliResult {
        let report = match RobloxStudioPaths::new() {
            Ok(paths) => {
                let executable = PathStatus::from_path(paths.exe());
                let content = PathStatus::from_path(paths.content());
                let built_in_plugins = PathStatus::from_path(paths.built_in_plugins());
                let user_plugins = PathStatus::from_path(paths.user_plugins());
                let ok = executable.exists && content.exists && built_in_plugins.exists;

                DoctorReport {
                    ok,
                    os: env::consts::OS,
                    arch: env::consts::ARCH,
                    executable: Some(executable),
                    content: Some(content),
                    built_in_plugins: Some(built_in_plugins),
                    user_plugins: Some(user_plugins),
                    error: None,
                }
            }
            Err(error) => DoctorReport {
                ok: false,
                os: env::consts::OS,
                arch: env::consts::ARCH,
                executable: None,
                content: None,
                built_in_plugins: None,
                user_plugins: None,
                error: Some(error.to_string()),
            },
        };

        if self.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!("Roblox Studio diagnostics:");
            println!();
            println!(
                "- Status:           {}",
                if report.ok { "ok" } else { "issue detected" }
            );
            println!("- Platform:         {} ({})", report.os, report.arch);

            if let Some(executable) = &report.executable {
                print_path_status("Executable", executable);
            }
            if let Some(content) = &report.content {
                print_path_status("Content", content);
            }
            if let Some(built_in_plugins) = &report.built_in_plugins {
                print_path_status("Built-in Plugins", built_in_plugins);
            }
            if let Some(user_plugins) = &report.user_plugins {
                print_path_status("User Plugins", user_plugins);
            }
            if let Some(error) = &report.error {
                println!("- Error:            {error}");
            }
        }

        Ok(())
    }
}

impl PathStatus {
    fn from_path(path: &Path) -> Self {
        Self {
            path: path.display().to_string(),
            exists: path.exists(),
        }
    }
}

fn print_path_status(label: &str, status: &PathStatus) {
    let availability = if status.exists { "ok" } else { "missing" };
    println!("- {label:<16} {} ({availability})", status.path);
}

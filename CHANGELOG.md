<!-- markdownlint-disable MD023 -->
<!-- markdownlint-disable MD033 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

This release adds a background mode for the script running capabilities added in version `0.4.0`. It has been well-tested on macOS and does not yet work on Windows.

### Breaking Changes

- `RobloxStudioOpener::spawn` now returns a `RobloxStudioProcess` instead of a `std::process::Child`, since Roblox Studio is no longer a "normal" child process when opened in the background.

### Added

- Added `in_background` to `RobloxStudioOpener`, for running scripts in Roblox Studio without it ever showing a window or stealing focus. Currently only supported on macOS.
- Added `wait_timeout` and `stop` to `RobloxStudioProcess`, so that a script that never finishes can no longer leave Roblox Studio running forever.
- Added `--background` and `--timeout` options to the CLI.

## `0.4.0` - October 3rd, 2026

### Added

- Added support for the `RunScript` task, through the new `run_script`, `run_script_in_place`, and `run_script_in_file` methods on `RobloxStudioOpener`
- Added the `with_output_file` and `quit_after_execution` methods to `RobloxStudioOpener` for use with the `RunScript` task
- Added a new `spawn` method to `RobloxStudioOpener` that returns a handle to the Roblox Studio process, instead of detaching it
- Added a `run` command to the CLI for running Luau scripts in Roblox Studio
- Added a new `UnsupportedPlatform` variant to `RobloxStudioError`

### Fixed

- Fixed `RobloxStudioPaths::new` panicking on Linux - it now returns `RobloxStudioError::UnsupportedPlatform` instead
- Fixed compilation on platforms other than Windows, macOS, and Linux, which now also return `RobloxStudioError::UnsupportedPlatform`
- Fixed `RobloxStudioPaths::new` succeeding on macOS even when Roblox Studio is not installed - it now returns an error, same as on Windows

## `0.3.3` - June 1st, 2026

### Added

- Added output to CLI for the global settings file location
- Added a new `global_settings` accessor to `RobloxStudioPaths`

## `0.3.2` - March 15th, 2026

### Fixed

- Fixed Windows Studio launches failing under job-managed parent processes by retrying detached spawn without `CREATE_BREAKAWAY_FROM_JOB` when Windows denies that flag

## `0.3.1` - March 5th, 2026

### Fixed

- Fixed opening online places on Windows by using the Roblox Studio launcher when the edit-place task requires it

## `0.3.0` - March 5th, 2026

### Added

- Added a tiny CLI with commands for printing discovered Studio paths, opening local files and online places, and starting local test workflows

### Changed

- Roblox Studio is now spawned as a properly detached child process, making launches behave more reliably when invoked from other tooling

## `0.2.0` - April 18th, 2025

### Added

- Added the `RobloxStudioPaths` API for discovering the current Studio executable, content directory, and plugin directories
- Added Windows support for Studio path discovery
- Added support for customizing the server address and port used when launching local test servers and clients

### Changed

- Reworked the Studio launching implementation to be more robust, ditching the `opener` crate

### Fixed

- Fixed compilation issues on Windows

## `0.1.0` - July 5th, 2024

Initial release of `roblox-studio-utils`, providing a Rust API for opening local place files, opening online places, and starting local Roblox Studio test sessions.

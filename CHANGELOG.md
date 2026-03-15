<!-- markdownlint-disable MD023 -->
<!-- markdownlint-disable MD033 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

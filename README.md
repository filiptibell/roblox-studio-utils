<!-- markdownlint-disable MD033 -->

<h1 align="center"><code>roblox-studio-utils</code></h1>

<div align="center">
	<div>
		<a href="https://crates.io/crates/roblox-studio-utils">
			<img src="https://img.shields.io/crates/v/roblox-studio-utils.svg?label=Version" alt="Crate version" />
		</a>
		<a href="https://github.com/filiptibell/roblox-studio-utils/actions">
			<img src="https://shields.io/endpoint?url=https://badges.readysetplay.io/workflow/filiptibell/roblox-studio-utils/ci.yaml" alt="CI status" />
		</a>
		<a href="https://github.com/filiptibell/roblox-studio-utils/blob/main/LICENSE.txt">
			<img src="https://img.shields.io/github/license/filiptibell/roblox-studio-utils.svg?label=License&color=informational" alt="Crate license" />
		</a>
	</div>
</div>

<br/>

Control Roblox Studio from your terminal - or from Rust.

## Features

- 📜 Run Luau scripts in Roblox Studio, and wait for them to finish
- 🤫 Run them **_completely_** in the background - no windows, no stolen focus <sup>[1]</sup>
- ⏱️ Timeouts that are actually guaranteed, so Roblox Studio never gets stuck forever
- 🌎 Open local place files and online places with a single command
- 🧪 Start local test servers & clients for quick multiplayer testing
- 🔍 Find Roblox Studio, its plugins, and its settings, on both Windows and macOS

<sup>[1]</sup> Background mode is currently only supported on macOS.

## Installation

The CLI can be installed using [Rokit][rokit]:

```sh
rokit add filiptibell/roblox-studio-utils
```

The library can be added to any Rust project using `cargo`:

```sh
cargo add roblox-studio-utils
```

## Usage

### CLI

Run a script, wait for Roblox Studio to finish, and get its output:

```sh
roblox-studio-utils run my_script.luau --quit-after-execution --output-file output.log
```

Want to keep working while it runs? Add `--background`, and Roblox Studio stays _completely_ out of your way. <br/>
Add `--timeout` too, and it gets stopped after a minute, even if your script never finishes:

```sh
roblox-studio-utils run my_script.luau --quit-after-execution --background --timeout 60
```

Running `roblox-studio-utils --help` will give you a full overview of all available commands. <br/>
Running `roblox-studio-utils command-name --help` will give you full details about a _specific_ command.

<details> <summary> <b>Brief overview of available commands</b> </summary>

- `roblox-studio-utils run` - Runs a Luau script in a baseplate, a local place file, or an online place.
- `roblox-studio-utils open file` - Opens a local place file.
- `roblox-studio-utils open place` - Opens an online place.
- `roblox-studio-utils test server` - Starts a local test server.
- `roblox-studio-utils test client` - Starts a test client that connects to a running test server.
- `roblox-studio-utils test session` - Starts a local test server, together with clients.
- `roblox-studio-utils paths` - Prints where Roblox Studio and its files are located.
- `roblox-studio-utils doctor` - Checks your Roblox Studio installation for problems.

</details>

### Library

Everything the CLI does is also available as a library. Here's the same script, from Rust:

```rust
use std::time::Duration;

use roblox_studio_utils::RobloxStudioOpener;

let mut studio = RobloxStudioOpener::new()
    .run_script("my_script.luau")?
    .quit_after_execution()
    .in_background()
    .spawn()?;

// Give the script 45 seconds, then ask Roblox Studio to quit, and kill it after 15 more
if studio.wait_timeout(Duration::from_secs(45))?.is_none() {
    studio.stop(Duration::from_secs(15))?;
}
```

Check out the [examples](/examples/) directory for more.

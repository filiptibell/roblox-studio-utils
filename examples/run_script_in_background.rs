use std::time::Duration;

use roblox_studio_utils::RobloxStudioOpener;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let script_path = "my_script.luau";
    let output_path = "my_script_output.log";

    if !RobloxStudioOpener::supports_background() {
        return Err("opening Roblox Studio in the background is not supported here".into());
    }

    // Roblox Studio never activates or shows any windows while it runs the script
    let mut studio = RobloxStudioOpener::new()
        .run_script(script_path)?
        .with_output_file(output_path)?
        .quit_after_execution()
        .in_background()
        .spawn()?;

    // Make sure that Roblox Studio exits within a minute, even if the script never finishes
    let exit = match studio.wait_timeout(Duration::from_secs(45))? {
        Some(exit) => exit,
        None => studio.stop(Duration::from_secs(15))?,
    };

    println!("Roblox Studio exited with {exit}");
    println!("{}", std::fs::read_to_string(output_path)?);

    Ok(())
}

use roblox_studio_utils::RobloxStudioOpener;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let script_path = "my_script.luau";
    let output_path = "my_script_output.log";

    let status = RobloxStudioOpener::new()
        .run_script(script_path)?
        .with_output_file(output_path)?
        .quit_after_execution()
        .spawn()?
        .wait()?;

    println!("Roblox Studio exited with {status}");
    println!("{}", std::fs::read_to_string(output_path)?);

    Ok(())
}

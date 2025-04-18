use roblox_studio_utils::RobloxStudioPaths;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths = RobloxStudioPaths::new()?;

    println!("Paths to current Roblox Studio installation:");
    println!();
    println!("- Executable:       {}", paths.exe().display());
    println!("- Content:          {}", paths.content().display());
    println!("- Built-in Plugins: {}", paths.built_in_plugins().display());
    println!("- User Plugins:     {}", paths.user_plugins().display());

    Ok(())
}

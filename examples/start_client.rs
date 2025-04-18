use roblox_studio_utils::RobloxStudioOpener;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // NOTE: A server must already be running for this to work

    RobloxStudioOpener::new().start_client().run()?;

    Ok(())
}

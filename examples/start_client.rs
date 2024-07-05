use roblox_studio_utils::RobloxStudioOpener;

pub fn main() -> anyhow::Result<()> {
    // NOTE: A server must already be running for this to work

    RobloxStudioOpener::new().start_client().run()?;

    Ok(())
}

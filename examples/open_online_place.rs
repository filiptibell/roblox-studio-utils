use roblox_studio_utils::RobloxStudioOpener;

pub fn main() -> anyhow::Result<()> {
    let universe_id: u64 = 123456789;
    let place_id: u64 = 234567890;

    RobloxStudioOpener::new()
        .open_place(universe_id, place_id)
        .run()?;

    Ok(())
}
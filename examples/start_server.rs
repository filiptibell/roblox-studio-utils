use roblox_studio_utils::RobloxStudioOpener;

pub fn main() -> anyhow::Result<()> {
    let file_path = "my_place_file.rbxl";

    RobloxStudioOpener::new().start_server(file_path)?.run()?;

    Ok(())
}

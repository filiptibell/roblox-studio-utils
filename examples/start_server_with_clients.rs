use roblox_studio_utils::RobloxStudioOpener;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "my_place_file.rbxl";
    let num_clients = 4;

    RobloxStudioOpener::new()
        .start_server_with_clients(file_path, num_clients)?
        .run()?;

    Ok(())
}

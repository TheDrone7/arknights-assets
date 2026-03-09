mod errors;
mod models;
mod progress;
mod server;

use anyhow::{Context, Result};
pub use server::Server;
use std::path::Path;
use tokio::fs;

pub async fn download(server: Server, output_dir: &str) -> Result<()> {
    let base_path = Path::new(output_dir);
    let temp_path = base_path.join(".tmp");

    fs::create_dir_all(&base_path)
        .await
        .with_context(|| format!("Failed to create output directory at {:?}", base_path))?;
    println!("Output directory ready: {:?}", base_path);

    fs::create_dir_all(&temp_path)
        .await
        .with_context(|| format!("Failed to create temporary directory at {:?}", temp_path))?;
    println!("Temporary directory ready: {:?}", temp_path);

    println!("Initializing download from server: '{}'", server);

    let client = reqwest::Client::new();
    println!("Fetching version info for server: '{}'", server);

    let version = client
        .get(server.version_url())
        .send()
        .await?
        .error_for_status()?
        .json::<models::VersionInfo>()
        .await?;

    println!("Received version: {}", version.res_version);

    let version_file = base_path.join("version.json");
    let version_json = serde_json::to_string_pretty(&version)
        .with_context(|| "Failed to stringify version json")?;
    tokio::fs::write(&version_file, version_json)
        .await
        .with_context(|| format!("Failed to store version file at {:?}", version_file))?;
    println!("Saved version info to {:?}", version_file);

    println!(
        "Fetching hot update list for server: '{}' and version: '{}'",
        server, version.res_version
    );

    let hot_update_url = server.hot_update_url(&version.res_version);
    let update_list = client
        .get(&hot_update_url)
        .send()
        .await?
        .error_for_status()?
        .json::<models::HotUpdateList>()
        .await?;

    println!("Total files to download: {}", update_list.ab_infos.len());

    let metadata = progress::ProgressTracker::load(base_path).await?;
    let mut pending = Vec::new();

    for info in update_list.ab_infos {
        if !metadata.is_up_to_date(&info.name, &info.md5) {
            pending.push(info);
        }
    }

    println!("Pending downloads: {}", pending.len());
    let _error_logger = errors::ErrorLogger::init(base_path).await?;

    // TODO: Implement resumability
    // TODO: Implement streaming downloads
    // TODO: Implement error logging
    Ok(())
}

mod download;
mod errors;
mod models;
mod progress;
mod server;
mod ui;

pub use server::Server;

use anyhow::{Context, Result};
use futures::StreamExt;
use std::path::Path;
use tokio::fs;

pub async fn download(server: Server, output_dir: &str, threads: usize) -> Result<()> {
    let base_path = Path::new(output_dir);
    let temp_path = base_path.join(".tmp");

    println!("[1/6] Preparing output and temporary directories.");
    fs::create_dir_all(&base_path)
        .await
        .with_context(|| format!("Failed to create output directory at {:?}", base_path))?;
    fs::create_dir_all(&temp_path)
        .await
        .with_context(|| format!("Failed to create temporary directory at {:?}", temp_path))?;

    let client = reqwest::Client::new();
    println!("[2/6] Fetching version info for server: '{}'", server);
    let version = client
        .get(server.version_url())
        .send()
        .await?
        .error_for_status()?
        .json::<models::VersionInfo>()
        .await?;

    let version_file = base_path.join("version.json");
    let version_json = serde_json::to_string_pretty(&version)
        .with_context(|| "Failed to stringify version json")?;
    tokio::fs::write(&version_file, version_json)
        .await
        .with_context(|| format!("Failed to store version file at {:?}", version_file))?;

    println!(
        "[3/6] Fetching hot update list for server: '{}' and version: '{}'",
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

    let total_files = update_list.ab_infos.len();
    println!("[4/6] Total files to download: {}", total_files);

    let ui = ui::DownloadUi::new(total_files as u64);
    let mut tracker = progress::ProgressTracker::load(base_path).await?;
    let mut pending = Vec::new();

    for info in update_list.ab_infos {
        if !tracker.is_up_to_date(&info.name, &info.md5) {
            pending.push(info);
        } else {
            ui.inc_main();
        }
    }

    let error_logger = errors::ErrorLogger::init(base_path).await?;

    let mut stream = futures::stream::iter(pending)
        .map(|info| {
            let pb = ui.add_download_bar(&info.name);
            let temp_file = temp_path.join(&info.name);
            let final_file = base_path.join(&info.name);
            let client_ref = client.clone();
            let res_version = version.res_version.clone();

            async move {
                let res = download::stream_download(
                    &client_ref,
                    &server,
                    &res_version,
                    &info.name,
                    &temp_file,
                    &final_file,
                )
                .await;
                pb.finish_and_clear();
                (info, res)
            }
        })
        .buffer_unordered(threads);

    while let Some((info, result)) = stream.next().await {
        match result {
            Ok(_) => {
                tracker.mark_completed(info.name, info.md5).await?;
            }
            Err(e) => {
                error_logger
                    .log_error(format!("{}: {:?}", &info.name, e))
                    .await;
            }
        }
        ui.inc_main();
    }

    ui.finish();

    if error_logger.has_errors().await {
        println!(
            "[5/6] Finished with errors. Check log: {:?}",
            error_logger.path()
        );
    } else {
        println!("[5/6] Successfully finished downloading assets.");
    }

    println!("[6/6] Cleaning up temporary directories.");
    fs::remove_dir_all(&temp_path)
        .await
        .with_context(|| format!("Failed to delete temporary directories: {:?}", &temp_path))?;

    Ok(())
}

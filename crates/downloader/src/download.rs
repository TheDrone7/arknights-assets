use crate::Server;
use anyhow::{Context, Result};
use futures::StreamExt;
use regex::Regex;
use reqwest::Client;
use std::path::Path;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

pub async fn stream_download(
    client: &Client,
    server: &Server,
    res_version: &str,
    filename: &str,
    tmp_path: &Path,
    final_path: &Path,
) -> Result<()> {
    let re = Regex::new(r"\.[^.]*$").unwrap();
    let dat_path = re.replace(filename, ".dat");
    let url_path = dat_path.replace("/", "_").replace("#", "__");

    let url = server.asset_url(res_version, &url_path);

    if let Some(parent) = tmp_path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create temp dir: {:?}", &parent))?;
    }

    if let Some(parent) = final_path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create final dir: {:?}", &parent))?;
    }

    let mut file = File::create(&tmp_path)
        .await
        .with_context(|| format!("Failed to create temp file at: {:?}", &tmp_path))?;
    let res = client
        .get(&url)
        .header("User-Agent", "BestHTTP")
        .send()
        .await
        .with_context(|| format!("Failed to request file from URL: {:?}", &url))?
        .error_for_status()?;

    let mut stream = res.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed reading stream chunk")?;
        file.write_all(&chunk)
            .await
            .context("Failed to write to file")?;
    }

    file.flush().await?;

    fs::rename(tmp_path, final_path)
        .await
        .context("Failed moving file")?;
    Ok(())
}

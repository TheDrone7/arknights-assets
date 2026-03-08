use anyhow::Result;

pub async fn download(server: &str, output_dir: &str) -> Result<()> {
    println!("Initializing downloader from server: '{}'", server);
    println!("Output directory: '{}'", output_dir);
    println!("... Fetching version list");
    println!("... Downloading files");
    Ok(())
}

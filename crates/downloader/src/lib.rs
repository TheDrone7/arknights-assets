mod server;

use anyhow::Result;
pub use server::Server;

pub async fn download(server: Server, output_dir: &str) -> Result<()> {
    println!("Initializing downloader from server: '{}'", server);
    println!("Output directory: '{}'", output_dir);
    println!("... Fetching version list");
    println!("... Downloading files");
    Ok(())
}

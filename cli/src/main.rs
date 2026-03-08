use ak_downloader::download;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "arknights-cli",
    version,
    about = "A CLI tool for downloading and processing Arknights assets."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Download {
        #[arg(short, long, default_value = "en")]
        server: String,

        #[arg(short, long, default_value = "./data/raw")]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Download { server, output } => {
            println!("Starting downloader...");
            download(server, output).await?;
            println!("Download completed.");
        }
    }

    Ok(())
}

use ak_downloader::{Server, download};
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

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
        #[arg(
            short,
            long,
            default_value = "en",
            help = "The server to download assets from"
        )]
        server: CliServer,

        #[arg(
            short,
            long,
            default_value = "./data/raw",
            help = "The directory where the assets should be stored"
        )]
        output: String,

        #[arg(
            short,
            long,
            default_value_t = 1,
            help = "The number of files to download concurrently"
        )]
        threads: usize,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliServer {
    En,
    Cn,
    Bl,
    Jp,
    Kr,
    Tw,
}

impl From<CliServer> for Server {
    fn from(cli_server: CliServer) -> Self {
        match cli_server {
            CliServer::En => Server::En,
            CliServer::Cn => Server::Cn,
            CliServer::Bl => Server::Bl,
            CliServer::Jp => Server::Jp,
            CliServer::Kr => Server::Kr,
            CliServer::Tw => Server::Tw,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Download {
            server,
            output,
            threads,
        } => {
            println!("Starting downloader...");
            download(server.into(), &output, threads).await?;
            println!("Download completed.");
        }
    }

    Ok(())
}

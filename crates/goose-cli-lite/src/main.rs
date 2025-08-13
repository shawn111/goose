use anyhow::Result;
use clap::{Parser, Subcommand};

mod handlers;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Display Goose information
    Info {
        /// Show verbose information including current configuration
        #[arg(short, long, help = "Show verbose information including config.yaml")]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Info { verbose }) => {
            handlers::handle_info(verbose).await?;
        }
        None => {
            println!("Welcome to goose-cli-lite! Use --help for available commands.");
        }
    }

    Ok(())
}
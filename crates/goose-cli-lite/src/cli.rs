use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Display Goose information
    Info {
        /// Show verbose information including current configuration
        #[arg(short, long, help = "Show verbose information including config.yaml")]
        verbose: bool,
    },
}

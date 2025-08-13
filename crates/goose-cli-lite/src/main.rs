use anyhow::Result;
use clap::Parser;
use goose_cli_lite::{cli, commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        Some(cli::Command::Info { verbose }) => {
            commands::info::handle_info(verbose).await?;
        }
        None => {
            println!("starting session | provider: gemini-cli model: gemini-2.5-flash");
            println!("    logging to /data/data/com.termux/files/home/.local/share/goose/sessions/20250813_134303.jsonl");
            println!("    working directory: /data/data/com.termux/files/home");
            println!("");
            println!("Goose is running! Enter your instructions, or try asking what goose can do.");
            println!("");
            println!("Context: ○○○○○○○○○○ 0% (0/1000000 tokens)");
            println!("( O)> Press Enter to send, Ctrl-J for new line");
        }
    }

    Ok(())
}
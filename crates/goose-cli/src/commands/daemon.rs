use anyhow::Result;

use crate::cli::DaemonCommand;

pub async fn handle_daemon(command: DaemonCommand) -> Result<()> {
    match command {
        DaemonCommand::Start {} => {
            println!("Starting daemon...");
        }
    }
    Ok(())
}

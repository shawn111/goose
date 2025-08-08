use anyhow::Result;

use crate::cli::SessionSidecarCommand;

pub async fn handle_session_sidecar(command: SessionSidecarCommand) -> Result<()> {
    match command {
        SessionSidecarCommand::Enter {} => {
            println!("Entering Goose session...");
        }
        SessionSidecarCommand::Exit {} => {
            println!("Exiting Goose session...");
        }
        SessionSidecarCommand::Status {} => {
            println!("Checking Goose session status...");
        }
    }
    Ok(())
}

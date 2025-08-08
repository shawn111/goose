use anyhow::Result;

pub fn handle_history_add(session_id: String, command: String, cwd: String) -> Result<()> {
    println!("session_id: {}", session_id);
    println!("command: {}", command);
    println!("cwd: {}", cwd);
    Ok(())
}

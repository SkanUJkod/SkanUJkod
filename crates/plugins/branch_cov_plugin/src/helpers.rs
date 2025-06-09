// Simplified helpers module for branch coverage plugin

pub mod go_utils {
    use anyhow::Result;

    pub fn check_go_installation() -> Result<()> {
        // Simplified check - just verify go command exists
        match std::process::Command::new("go").arg("version").output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    anyhow::bail!("Go command failed")
                }
            }
            Err(_) => {
                anyhow::bail!("Go command not found. Please install Go.")
            }
        }
    }
}

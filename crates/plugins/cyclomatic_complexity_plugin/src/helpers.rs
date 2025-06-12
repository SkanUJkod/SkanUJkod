// Simplified helpers module for cyclomatic complexity plugin
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

impl ComplexityLevel {
    pub fn from_complexity(complexity: usize) -> Self {
        match complexity {
            1..=5 => Self::Low,
            6..=10 => Self::Medium,
            11..=20 => Self::High,
            _ => Self::VeryHigh,
        }
    }
}

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

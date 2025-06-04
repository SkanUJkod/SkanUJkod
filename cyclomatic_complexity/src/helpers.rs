use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Complexity level classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ComplexityLevel {
    Low,      // 1-5
    Moderate, // 6-10
    High,     // 11-20
    VeryHigh, // >20
}

impl std::fmt::Display for ComplexityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplexityLevel::Low => write!(f, "Low"),
            ComplexityLevel::Moderate => write!(f, "Moderate"),
            ComplexityLevel::High => write!(f, "High"),
            ComplexityLevel::VeryHigh => write!(f, "Very High"),
        }
    }
}

/// Helper utilities for Go projects
pub mod go_utils {
    use super::*;

    /// Check if Go is installed on the system
    pub fn check_go_installation() -> Result<String> {
        let output = Command::new("go")
            .arg("version")
            .output()
            .context("Failed to run 'go version'. Is Go installed?")?;

        if !output.status.success() {
            anyhow::bail!("Go is not properly installed");
        }

        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.trim().to_string())
    }

    /// Check if a path is a Go test file
    pub fn is_test_file(path: &Path) -> bool {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with("_test.go"))
            .unwrap_or(false)
    }
}

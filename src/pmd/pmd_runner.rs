use std::path::Path;
use std::process::Command;
use std::io;
use std::fmt;

#[derive(Debug)]
pub enum PmdError {
    IoError(io::Error),
    PmdFailed(String),
}

impl fmt::Display for PmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PmdError::IoError(e) => write!(f, "IO error: {}", e),
            PmdError::PmdFailed(msg) => write!(f, "PMD failed: {}", msg),
        }
    }
}

impl std::error::Error for PmdError {}

impl From<io::Error> for PmdError {
    fn from(err: io::Error) -> Self {
        PmdError::IoError(err)
    }
}

#[derive(Debug)]
pub struct PmdRunner {
    pub pmd_path: String,
}

impl PmdRunner {
    pub fn analyze_duplicates<P: AsRef<Path>>(
        &self,
        path: P,
        min_tokens: usize,
        is_directory: bool,
    ) -> Result<String, PmdError> {
        let path_ref = path.as_ref();
        let path_str = path_ref.to_str().ok_or_else(|| {
            PmdError::PmdFailed("Invalid path: not valid UTF-8".to_string())
        })?;

        if !is_directory {
            if let Some(ext) = path_ref.extension().and_then(|e| e.to_str()) {
                if ext.to_lowercase() != "go" {
                    eprintln!("Skipping non-Go file (client-side filter): {}", path_str);
                    return Ok(String::new());
                }
            } else {
                eprintln!("Skipping file without extension (client-side filter): {}", path_str);
                return Ok(String::new());
            }
        }

        let mut cmd_args = vec![
            "cpd".to_string(),
            "--minimum-tokens".to_string(),
            min_tokens.to_string(),
            "--language".to_string(),
            "go".to_string(),
            "--format".to_string(),
            "text".to_string(),
        ];

        if is_directory {
            cmd_args.push("--dir".to_string());
            cmd_args.push(path_str.to_string());
        } else {
            cmd_args.push(path_str.to_string());
        }
        
        let output = Command::new(&self.pmd_path)
            .args(&cmd_args)
            .output()?;

        let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

        let cpd_found_duplicates_exit_code = Some(4); 
        let is_cpd_success_with_findings = output.status.code() == cpd_found_duplicates_exit_code && !stdout_str.is_empty();

        if !output.status.success() && !is_cpd_success_with_findings {
            let mut error_message = format!(
                "PMD command execution failed. Status: {}.\nPMD Executable: '{}'\nArguments: {:?}\n",
                output.status, self.pmd_path, cmd_args
            );
            if !stdout_str.is_empty() {
                error_message.push_str(&format!("Stdout (on error):\n{}\n", stdout_str));
            }
            if !stderr_str.is_empty() {
                error_message.push_str(&format!("Stderr (on error):\n{}\n", stderr_str));
            }
            return Err(PmdError::PmdFailed(error_message.trim().to_string()));
        }

        if !stderr_str.is_empty() {
             eprintln!(
                "PMD Info/Warnings (stderr for '{}', exit_code: {:?}):\n{}",
                path_str, output.status.code(), stderr_str.trim()
            );
        }
        
        Ok(stdout_str)
    }
}

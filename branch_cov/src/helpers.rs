use crate::ProjectBranchCoverage;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone)]
pub struct BranchCoverageConfig {
    pub min_coverage: f64,
    pub exclude_functions: HashSet<String>,
    pub exclude_patterns: Vec<String>,
    pub fail_on_low_coverage: bool,
    pub verbose: bool,
    pub include_test_files: bool,
}

impl Default for BranchCoverageConfig {
    fn default() -> Self {
        Self {
            min_coverage: 80.0,
            exclude_functions: HashSet::new(),
            exclude_patterns: vec!["*_test.go".to_string(), "vendor/*".to_string()],
            fail_on_low_coverage: true,
            verbose: false,
            include_test_files: false,
        }
    }
}

pub struct BranchCoverageAnalyzer {
    config: BranchCoverageConfig,
}

impl BranchCoverageAnalyzer {
    pub fn new(config: BranchCoverageConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(BranchCoverageConfig::default())
    }

    pub fn analyze(&self, project_path: &Path) -> Result<ProjectBranchCoverage> {
        let options = crate::BranchCoverageOptions {
            verbose: self.config.verbose,
            include_test_files: self.config.include_test_files,
            min_coverage_threshold: self.config.min_coverage,
            fail_on_low_coverage: self.config.fail_on_low_coverage,
            exclude_patterns: self.config.exclude_patterns.clone(),
            simulate_coverage: false,
            test_args: Vec::new(),
            fail_on_error: false,
            timeout_seconds: 30,
        };

        crate::analyze_branch_coverage_with_options(project_path, &options)
    }

    pub fn set_min_coverage(&mut self, min_coverage: f64) {
        self.config.min_coverage = min_coverage;
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.config.verbose = verbose;
    }

    pub fn add_exclude_pattern(&mut self, pattern: String) {
        self.config.exclude_patterns.push(pattern);
    }

    pub fn exclude_function(&mut self, function_name: String) {
        self.config.exclude_functions.insert(function_name);
    }
}

pub mod export {
    use super::*;

    pub fn to_json(coverage: &ProjectBranchCoverage) -> Result<String> {
        serde_json::to_string_pretty(coverage)
            .context("Failed to serialize branch coverage to JSON")
    }

    pub fn to_csv(coverage: &ProjectBranchCoverage) -> String {
        let mut csv = String::new();
        csv.push_str("Function,File,TotalBranches,CoveredBranches,CoveragePercentage\n");

        for (func_name, func_coverage) in &coverage.functions {
            csv.push_str(&format!(
                "{},{},{},{},{:.2}\n",
                func_name,
                func_coverage.file_path,
                func_coverage.total_branches,
                func_coverage.covered_branches,
                func_coverage.coverage_percentage
            ));
        }

        csv
    }

    pub fn ci_export(coverage: &ProjectBranchCoverage, threshold: f64) -> i32 {
        if coverage.overall_coverage_percentage >= threshold {
            0 // Success
        } else {
            1 // Failure
        }
    }

    pub fn save_report(
        coverage: &ProjectBranchCoverage,
        output_path: &Path,
        format: &str,
    ) -> Result<()> {
        let content = match format {
            "json" => to_json(coverage)?,
            "csv" => to_csv(coverage),
            _ => anyhow::bail!("Unsupported format: {}", format),
        };

        fs::write(output_path, content).context("Failed to write coverage report to file")?;

        Ok(())
    }
}

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
    pub fn is_test_file(path: &str) -> bool {
        path.ends_with("_test.go")
    }

    /// Check if a path should be excluded based on patterns
    pub fn should_exclude_file(path: &str, exclude_patterns: &[String]) -> bool {
        for pattern in exclude_patterns {
            if path.contains(pattern.trim_matches('*')) {
                return true;
            }
        }
        false
    }

    /// Get all Go files in a directory (recursively)
    pub fn find_go_files(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut go_files = Vec::new();

        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    go_files.extend(find_go_files(&path)?);
                } else if let Some(ext) = path.extension() {
                    if ext == "go" {
                        go_files.push(path);
                    }
                }
            }
        }

        Ok(go_files)
    }

    /// Check if the project has go.mod (is a Go module)
    pub fn is_go_module(project_path: &Path) -> bool {
        project_path.join("go.mod").exists()
    }

    /// Initialize a Go module if it doesn't exist
    pub fn ensure_go_module(project_path: &Path) -> Result<()> {
        if !is_go_module(project_path) {
            let output = Command::new("go")
                .arg("mod")
                .arg("init")
                .arg("test-module")
                .current_dir(project_path)
                .output()
                .context("Failed to initialize Go module")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to initialize Go module: {}", stderr);
            }
        }
        Ok(())
    }
}

use crate::ProjectCoverage;
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize)]
pub struct FunctionCoverage {
    pub total_statements: usize,
    pub covered_statements: usize,
    pub coverage_percentage: f64,
    pub uncovered_blocks: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct CoverageConfig {
    pub min_coverage: f64,
    pub exclude_functions: HashSet<String>,
    pub exclude_patterns: Vec<String>,
    pub generate_html: bool,
    pub fail_on_low_coverage: bool,
    pub verbose: bool,
}

impl Default for CoverageConfig {
    fn default() -> Self {
        Self {
            min_coverage: 80.0,
            exclude_functions: HashSet::new(),
            exclude_patterns: vec!["*_test.go".to_string(), "vendor/*".to_string()],
            generate_html: false,
            fail_on_low_coverage: true,
            verbose: false,
        }
    }
}

pub struct CoverageAnalyzer {
    config: CoverageConfig,
}

impl CoverageAnalyzer {
    pub fn new(config: CoverageConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(CoverageConfig::default())
    }

    pub fn analyze(&self, project_path: &Path) -> Result<ProjectCoverage> {
        if self.config.verbose {
            println!("Analyzing project at: {}", project_path.display());
        }

        let coverage = crate::analyze_statement_coverage(project_path)?;

        if self.config.fail_on_low_coverage && coverage.overall_coverage < self.config.min_coverage
        {
            anyhow::bail!(
                "Coverage {:.1}% is below the minimum threshold {:.1}%",
                coverage.overall_coverage,
                self.config.min_coverage
            );
        }

        Ok(coverage)
    }
}

pub mod export {
    use super::*;

    pub fn to_json(coverage: &ProjectCoverage) -> Result<String> {
        serde_json::to_string_pretty(coverage).context("Failed to serialize coverage data")
    }

    pub fn to_csv(coverage: &ProjectCoverage) -> String {
        let mut csv = String::from("Function,Total Statements,Covered Statements,Coverage %\n");

        let mut functions: Vec<_> = coverage.functions.iter().collect();
        functions.sort_by_key(|(name, _)| name.as_str());

        for (name, func_cov) in functions {
            csv.push_str(&format!(
                "{},{},{},{:.1}\n",
                name,
                func_cov.total_statements,
                func_cov.covered_statements,
                func_cov.coverage_percentage
            ));
        }

        csv.push_str(&format!(
            "TOTAL,{},{},{:.1}\n",
            coverage.total_statements, coverage.covered_statements, coverage.overall_coverage
        ));

        csv
    }

    pub fn ci_export(coverage: &ProjectCoverage, threshold: f64) -> i32 {
        println!(
            "::set-output name=coverage::{:.1}",
            coverage.overall_coverage
        );
        println!(
            "::set-output name=total_statements::{}",
            coverage.total_statements
        );
        println!(
            "::set-output name=covered_statements::{}",
            coverage.covered_statements
        );

        if coverage.overall_coverage < threshold {
            eprintln!(
                "Coverage {:.1}% is below threshold {:.1}%",
                coverage.overall_coverage, threshold
            );
            1
        } else {
            0
        }
    }
}

pub mod go_utils {
    use super::*;

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

    pub fn find_go_files(project_path: &Path) -> Result<Vec<PathBuf>> {
        let mut go_files = Vec::new();

        fn visit_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        if !name_str.starts_with('.')
                            && name_str != "vendor"
                            && name_str != "testdata"
                        {
                            visit_dir(&path, files)?;
                        }
                    }
                } else if path.extension().and_then(|s| s.to_str()) == Some("go") {
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        if !name_str.ends_with("_test.go") && !name_str.contains("_instrumented") {
                            files.push(path);
                        }
                    }
                }
            }
            Ok(())
        }

        visit_dir(project_path, &mut go_files)?;
        Ok(go_files)
    }
}
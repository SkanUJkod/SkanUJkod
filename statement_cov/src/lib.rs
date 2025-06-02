use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

use cfg::ast::parse_project;
use cfg::cfg::{build_cfgs_for_file, ControlFlowGraph};

pub mod helpers;
pub mod instrumentation;

/// Structure representing coverage results for a single function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCoverage {
    pub total_statements: usize,
    pub covered_statements: usize,
    pub coverage_percentage: f64,
    pub uncovered_blocks: Vec<usize>,
}

/// Structure representing coverage results for the entire project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCoverage {
    pub functions: HashMap<String, FunctionCoverage>,
    pub total_statements: usize,
    pub covered_statements: usize,
    pub overall_coverage: f64,
}

/// Main entry point for analyzing statement coverage
///
/// # Arguments
/// * `project_path` - Path to the Go project directory
///
/// # Returns
/// * `ProjectCoverage` - Complete coverage analysis results
pub fn analyze_statement_coverage(project_path: &Path) -> Result<ProjectCoverage> {
    // Validate project path
    if !project_path.exists() {
        anyhow::bail!("Project path does not exist: {}", project_path.display());
    }

    // Check if Go is installed
    helpers::go_utils::check_go_installation().context("Go installation check failed")?;

    // Create temporary directory for instrumented code
    let temp_dir = TempDir::new().context("Failed to create temp directory")?;
    let temp_path = temp_dir.path();

    println!("Analyzing project: {}", project_path.display());

    // Step 1: Parse the project and build CFGs
    let (fset, objs, files) = parse_project(project_path).context("Failed to parse Go project")?;

    let mut cfgs = HashMap::new();
    for pf in &files {
        let file_cfgs = build_cfgs_for_file(&fset, &objs, &pf.ast);
        cfgs.extend(file_cfgs);
    }

    if cfgs.is_empty() {
        anyhow::bail!("No functions found in the project");
    }

    println!("Found {} functions to analyze", cfgs.len());

    // Step 2: Generate instrumented code
    instrumentation::generate_instrumented_project(&cfgs, project_path, temp_path)?;

    // Step 3: Run tests and collect coverage data
    let coverage_data = run_tests_and_collect_coverage(temp_path)?;

    // Step 4: Analyze coverage results
    let coverage = calculate_coverage(&cfgs, &coverage_data)?;

    // Clean up is automatic when temp_dir goes out of scope

    Ok(coverage)
}

/// Run tests and collect coverage data
fn run_tests_and_collect_coverage(project_path: &Path) -> Result<HashMap<String, Vec<usize>>> {
    println!("Running tests...");

    // First, ensure go.mod exists or create one
    let go_mod_path = project_path.join("go.mod");
    if !go_mod_path.exists() {
        // Create a basic go.mod
        let mod_content = "module coverage_test\n\ngo 1.19\n";
        fs::write(&go_mod_path, mod_content)?;
    }

    // Run go test
    let output = Command::new("go")
        .arg("test")
        .arg("-v")
        .arg("./...")
        .current_dir(project_path)
        .env("GO111MODULE", "on")
        .output()
        .context("Failed to run go test")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("Test output:\n{}", stdout);
        eprintln!("Test errors:\n{}", stderr);
        anyhow::bail!("Go tests failed");
    }

    println!("Tests completed successfully");

    // Read coverage data
    let coverage_file = project_path.join("coverage_data.json");
    if !coverage_file.exists() {
        anyhow::bail!("Coverage data file not found. Tests may not have run properly.");
    }

    let data = fs::read_to_string(&coverage_file).context("Failed to read coverage data")?;

    let coverage: HashMap<String, Vec<usize>> =
        serde_json::from_str(&data).context("Failed to parse coverage data")?;

    Ok(coverage)
}

/// Calculate coverage statistics
fn calculate_coverage(
    cfgs: &HashMap<String, ControlFlowGraph>,
    coverage_data: &HashMap<String, Vec<usize>>,
) -> Result<ProjectCoverage> {
    let mut functions = HashMap::new();
    let mut total_statements = 0;
    let mut covered_statements = 0;

    for (func_name, cfg) in cfgs {
        // Exclude entry and exit blocks from the total count
        // Assume exit is last and entry is first
        let total_blocks = cfg.blocks.len().saturating_sub(2); // Subtract exit + entry

        // Count covered blocks excluding the entry block
        let covered_blocks = coverage_data
            .get(func_name)
            .map(|blocks| blocks.iter().filter(|&&id| id != cfg.entry).count())
            .unwrap_or(0);

        // Find uncovered blocks
        let covered_set: HashSet<usize> = coverage_data
            .get(func_name)
            .map(|blocks| blocks.iter().copied().collect())
            .unwrap_or_default();

        let uncovered_blocks: Vec<usize> = cfg
            .blocks
            .keys()
            .filter(|&id| *id != cfg.entry && !covered_set.contains(id)) // Exclude entry from uncovered
            .filter(|&&id| id != *cfg.blocks.keys().max().unwrap_or(&0)) // Exclude exit block if it is highest ID
            .copied()
            .collect();

        let coverage_percentage = if total_blocks > 0 {
            (covered_blocks as f64 / total_blocks as f64) * 100.0
        } else {
            100.0 // Empty functions are considered 100% covered
        };

        functions.insert(
            func_name.clone(),
            FunctionCoverage {
                total_statements: total_blocks,
                covered_statements: covered_blocks,
                coverage_percentage,
                uncovered_blocks,
            },
        );

        total_statements += total_blocks;
        covered_statements += covered_blocks;
    }

    let overall_coverage = if total_statements > 0 {
        (covered_statements as f64 / total_statements as f64) * 100.0
    } else {
        100.0
    };

    Ok(ProjectCoverage {
        functions,
        total_statements,
        covered_statements,
        overall_coverage,
    })
}

/// Helper function to print coverage report
pub fn print_coverage_report(coverage: &ProjectCoverage) {
    println!("\n╔══════════════════════════════════════╗");
    println!("║     Statement Coverage Report        ║");
    println!("╚══════════════════════════════════════╝\n");

    // Sort functions by name for consistent output
    let mut functions: Vec<_> = coverage.functions.iter().collect();
    functions.sort_by_key(|(name, _)| name.as_str());

    for (func_name, func_coverage) in functions {
        let symbol = if func_coverage.coverage_percentage >= 80.0 {
            "✓"
        } else if func_coverage.coverage_percentage >= 50.0 {
            "⚠"
        } else {
            "✗"
        };

        println!("{} Function: {}", symbol, func_name);
        println!(
            "  Coverage: {:.1}% ({}/{})",
            func_coverage.coverage_percentage,
            func_coverage.covered_statements,
            func_coverage.total_statements
        );

        if !func_coverage.uncovered_blocks.is_empty() && func_coverage.uncovered_blocks.len() <= 10
        {
            println!("  Uncovered blocks: {:?}", func_coverage.uncovered_blocks);
        } else if !func_coverage.uncovered_blocks.is_empty() {
            println!(
                "  Uncovered blocks: {} blocks not covered",
                func_coverage.uncovered_blocks.len()
            );
        }
        println!();
    }

    println!("─────────────────────────────────────");
    println!(
        "Overall Coverage: {:.1}% ({}/{})",
        coverage.overall_coverage, coverage.covered_statements, coverage.total_statements
    );

    if coverage.overall_coverage >= 80.0 {
        println!("✅ Good coverage!");
    } else if coverage.overall_coverage >= 50.0 {
        println!("⚠️  Coverage could be improved");
    } else {
        println!("❌ Low coverage - more tests needed");
    }
}

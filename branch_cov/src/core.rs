use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use tempfile::TempDir;
use cfg::{parse_project, build_cfgs_for_file};

use crate::helpers::go_utils;
use crate::instrumentation::{InstrumentationData, instrument_for_branch_coverage, modify_source_files};

/// Represents information about a branch in the code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub branch_id: String,
    pub line: usize,
    pub branch_type: String, // "if-then", "if-else", "for", "switch-case", etc.
    pub condition: String,
    pub is_covered: bool,
}

/// Details about an uncovered branch in the code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredBranch {
    pub branch_id: String,
    pub line: usize,
    pub branch_type: String,
    pub condition: String,
    pub file_path: String,
}

/// Represents branch coverage information for a single function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionBranchCoverage {
    pub total_branches: usize,
    pub covered_branches: usize,
    pub coverage_percentage: f64,
    pub branches: Vec<BranchInfo>,
    pub uncovered_branches: Vec<UncoveredBranch>,
    pub function_name: String,
    pub file_path: String,
}

/// Represents branch coverage information for an entire project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBranchCoverage {
    pub functions: std::collections::HashMap<String, FunctionBranchCoverage>,
    pub total_branches: usize,
    pub covered_branches: usize,
    pub overall_coverage_percentage: f64,
    pub files_analyzed: Vec<String>,
    pub uncovered_branches: Vec<UncoveredBranch>,
    pub test_output: Option<String>,
}

/// Options for branch coverage analysis
#[derive(Debug, Clone)]
pub struct BranchCoverageOptions {
    pub verbose: bool,
    pub include_test_files: bool,
    pub min_coverage_threshold: f64,
    pub fail_on_low_coverage: bool,
    pub exclude_patterns: Vec<String>,
    pub simulate_coverage: bool,
    pub test_args: Vec<String>,
    pub fail_on_error: bool,
    pub timeout_seconds: u64,
}

impl Default for BranchCoverageOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            include_test_files: false,
            min_coverage_threshold: 80.0,
            fail_on_low_coverage: false,
            exclude_patterns: vec!["*_test.go".to_string(), "vendor/*".to_string()],
            simulate_coverage: false,
            test_args: Vec::new(),
            fail_on_error: false,
            timeout_seconds: 30,
        }
    }
}

/// Main entry point for analyzing branch coverage
/// 
/// # Arguments
/// * `project_path` - Path to the Go project to analyze
/// 
/// # Returns
/// * `Result<ProjectBranchCoverage>` - Analysis results or error
pub fn analyze_branch_coverage(project_path: &Path) -> Result<ProjectBranchCoverage> {
    analyze_branch_coverage_with_options(project_path, &BranchCoverageOptions::default())
}

/// Analyze branch coverage with custom options
/// 
/// # Arguments
/// * `project_path` - Path to the Go project to analyze
/// * `options` - Custom analysis options
/// 
/// # Returns
/// * `Result<ProjectBranchCoverage>` - Analysis results or error
pub fn analyze_branch_coverage_with_options(
    project_path: &Path,
    options: &BranchCoverageOptions,
) -> Result<ProjectBranchCoverage> {
    // Ensure Go is available
    go_utils::check_go_installation().context("Go installation check failed")?;

    // Parse project to get CFGs
    let (fset, objs, files) = parse_project(project_path)
        .context("Failed to parse Go project")?;
    let mut cfgs = std::collections::HashMap::new();
    for pf in &files {
        cfgs.extend(build_cfgs_for_file(&fset, &objs, &pf.ast));
    }

    // Create temporary directory for instrumented code
    let temp_dir = TempDir::new().context("Failed to create temp dir for instrumentation")?;
    let temp_path = temp_dir.path();
    
    if options.verbose {
        println!("Instrumenting code for branch coverage in {}", temp_path.display());
    }

    // Instrument code for branch coverage analysis
    let inst_data = instrument_for_branch_coverage(
        project_path,
        temp_path,
        &cfgs,
        &fset,
        &objs,
        options,
    )?;
    modify_source_files(temp_path, &inst_data)?;

    // Run tests and collect coverage data
    let (covered_branches, test_output) = run_tests_and_collect_coverage(
        temp_path, 
        options,
        &inst_data
    )?;

    // Process coverage data and generate report
    let result = calculate_branch_coverage(&inst_data, &covered_branches, test_output)?;
    
    if options.verbose {
        println!("Branch coverage analysis complete");
    }

    // Check coverage threshold
    if options.fail_on_low_coverage && result.overall_coverage_percentage < options.min_coverage_threshold {
        anyhow::bail!(
            "Branch coverage is below minimum threshold: {:.1}% < {:.1}%",
            result.overall_coverage_percentage,
            options.min_coverage_threshold
        );
    }

    Ok(result)
}

/// Run tests on the instrumented code and collect branch coverage data
fn run_tests_and_collect_coverage(
    project_path: &Path,
    options: &BranchCoverageOptions,
    inst_data: &InstrumentationData,
) -> Result<(HashSet<String>, Option<String>)> {
    if options.verbose {
        println!("🧪 Running tests to collect branch coverage...");
    }

    // First build the project to make sure it compiles
    let build_output = Command::new("go")
        .arg("build")
        .arg("./...")
        .current_dir(project_path)
        .env("GO111MODULE", "on")
        .output()
        .context("Failed to build instrumented project")?;

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        anyhow::bail!("Error: Build failed: {}", stderr);
    }

    // Prepare test command
    let mut test_cmd = Command::new("go");
    test_cmd
        .arg("test")
        .arg("-v")
        .arg("-timeout")
        .arg(format!("{}s", options.timeout_seconds))
        .arg("./...")
        .current_dir(project_path)
        .env("GO111MODULE", "on")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    // Add any additional test arguments
    for arg in &options.test_args {
        test_cmd.arg(arg);
    }

    // Run tests and collect output
    let mut child = test_cmd.spawn().context("Failed to spawn go test")?;
    let mut test_output = String::new();
    let mut covered_branches = HashSet::new();

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                // Parse branch coverage information from output
                if let Some(branch_id) = line.strip_prefix("BRANCH_COV:") {
                    covered_branches.insert(branch_id.trim().to_string());
                }
                
                if options.verbose {
                    println!("  {}", line);
                }
                
                test_output.push_str(&line);
                test_output.push('\n');
            }
        }
    }

    // Check if tests completed successfully
    let status = child.wait().context("Failed to wait for go test")?;
    if !status.success() && options.fail_on_error {
        anyhow::bail!("Tests failed. Enable verbose mode to see test output.");
    }

    // Check if coverage data was collected
    if covered_branches.is_empty() && inst_data.total_branches > 0 {
        if options.verbose {
            println!("Warning: No branch coverage data collected. Tests may have failed to run properly.");
        }
    }

    // If simulate_coverage is true, mark all branches as covered (useful for testing)
    if options.simulate_coverage {
        if options.verbose {
            println!("Simulating 100% branch coverage...");
        }
        
        for (branch_id, _) in &inst_data.branch_mapping {
            covered_branches.insert(branch_id.clone());
        }
    }

    Ok((covered_branches, Some(test_output)))
}

/// Calculate branch coverage based on execution data
fn calculate_branch_coverage(
    instrumentation_data: &InstrumentationData,
    covered_branches: &HashSet<String>,
    test_output: Option<String>,
) -> Result<ProjectBranchCoverage> {
    let mut functions = std::collections::HashMap::new();
    let mut total_branches = 0;
    let mut covered_branches_count = 0;
    let mut all_uncovered = Vec::new();
    
    // Process each function
    for (func_name, func_inst) in &instrumentation_data.instrumented_functions {
        let mut branches = Vec::new();
        let mut uncovered_branches = Vec::new();
        let mut func_total = 0;
        let mut func_covered = 0;
        
        // Process each branch in the function
        for branch in &func_inst.branches {
            let is_covered = covered_branches.contains(&branch.branch_id);
            func_total += 1;
            
            if is_covered {
                func_covered += 1;
            } else {
                // Add to uncovered branches
                uncovered_branches.push(UncoveredBranch {
                    branch_id: branch.branch_id.clone(),
                    line: branch.line,
                    branch_type: branch.branch_type.clone(),
                    condition: branch.condition.clone(),
                    file_path: func_inst.file_path.clone(),
                });
            }
            
            // Create branch info object
            branches.push(BranchInfo {
                branch_id: branch.branch_id.clone(),
                line: branch.line,
                branch_type: branch.branch_type.clone(),
                condition: branch.condition.clone(),
                is_covered,
            });
        }
        
        // Calculate coverage percentage
        let coverage_percentage = if func_total > 0 {
            (func_covered as f64 / func_total as f64) * 100.0
        } else {
            100.0
        };
        
        // Add function coverage
        functions.insert(
            func_name.clone(),
            FunctionBranchCoverage {
                function_name: func_name.clone(),
                file_path: func_inst.file_path.clone(),
                total_branches: func_total,
                covered_branches: func_covered,
                coverage_percentage,
                branches,
                uncovered_branches: uncovered_branches.clone(),
            },
        );
        
        // Update project totals
        total_branches += func_total;
        covered_branches_count += func_covered;
        all_uncovered.extend(uncovered_branches);
    }
    
    // Calculate overall coverage percentage
    let overall_coverage = if total_branches > 0 {
        (covered_branches_count as f64 / total_branches as f64) * 100.0
    } else {
        100.0
    };
    
    // Get list of files analyzed
    let files_analyzed: Vec<String> = instrumentation_data
        .instrumented_functions
        .values()
        .map(|f| f.file_path.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    
    Ok(ProjectBranchCoverage {
        functions,
        total_branches,
        covered_branches: covered_branches_count,
        overall_coverage_percentage: overall_coverage,
        files_analyzed,
        uncovered_branches: all_uncovered,
        test_output,
    })
}
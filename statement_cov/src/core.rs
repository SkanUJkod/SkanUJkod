use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use tempfile::TempDir;
use cfg::ast::parse_project;
use cfg::cfg::build_cfgs_for_file;

use crate::helpers::go_utils;
use crate::instrumentation;
use crate::instrumentation::InstrumentationData;

#[derive(Debug, Clone, Serialize)]
pub struct FunctionCoverage {
    pub total_statements: usize,
    pub covered_statements: usize,
    pub coverage_percentage: f64,
    pub uncovered_lines: Vec<usize>,
    pub uncovered_statements: Vec<usize>,
    pub uncovered_line_details: Vec<UncoveredLine>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UncoveredLine {
    pub line: usize,
    pub statement: String,
    pub stmt_type: String,
    pub stmt_ids: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectCoverage {
    pub files_analyzed: usize,
    pub total_statements: usize,
    pub covered_statements: usize,
    pub overall_coverage: f64,
    pub functions: HashMap<String, FunctionCoverage>,
    pub test_output: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CoverageOptions {
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

impl Default for CoverageOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            include_test_files: false,
            min_coverage_threshold: 80.0,
            fail_on_low_coverage: true,
            exclude_patterns: vec![],
            simulate_coverage: false,
            test_args: vec![],
            fail_on_error: true,
            timeout_seconds: 300,
        }
    }
}

pub fn analyze_statement_coverage(project_path: &Path) -> Result<ProjectCoverage> {
    analyze_statement_coverage_with_options(project_path, &CoverageOptions::default())
}

pub fn analyze_statement_coverage_with_options(
    project_path: &Path,
    options: &CoverageOptions,
) -> Result<ProjectCoverage> {
    if !project_path.exists() {
        anyhow::bail!("Project path does not exist: {}", project_path.display());
    }

    go_utils::check_go_installation().context("Go installation check failed")?;

    let temp_dir = TempDir::new().context("Failed to create temp directory")?;
    let temp_path = temp_dir.path();

    if options.verbose {
        println!("🔍 Analyzing project: {}", project_path.display());
        println!("📁 Temporary directory: {}", temp_path.display());
    }

    let (fset, objs, files) = parse_project(project_path).context("Failed to parse Go project")?;

    if files.is_empty() {
        anyhow::bail!("No Go files found in the project");
    }

    let mut cfgs = HashMap::new();
    for pf in &files {
        let file_cfgs = build_cfgs_for_file(&fset, &objs, &pf.ast);
        cfgs.extend(file_cfgs);
    }

    if cfgs.is_empty() {
        anyhow::bail!("No functions found in the project");
    }

    if options.verbose {
        println!("📊 Found {} functions to analyze", cfgs.len());
        println!("📄 Found {} Go files", files.len());
    }

    let instrumentation_data = instrumentation::generate_instrumented_project(
        &cfgs,
        &files,
        &fset,
        &objs,
        project_path,
        temp_path,
    ).context("Failed to instrument project")?;

    let (coverage_data, test_output) = run_tests_and_collect_coverage(temp_path, options)?;

    let mut coverage = calculate_statement_coverage(&instrumentation_data, &coverage_data)?;
    coverage.files_analyzed = files.len();
    coverage.test_output = test_output;

    Ok(coverage)
}

fn run_tests_and_collect_coverage(
    project_path: &Path,
    options: &CoverageOptions,
) -> Result<(HashMap<String, Vec<usize>>, Option<String>)> {
    if options.verbose {
        println!("🧪 Running tests...");
    }

    let build_output = Command::new("go")
        .arg("build")
        .arg("./...")
        .current_dir(project_path)
        .env("GO111MODULE", "on")
        .output()
        .context("Failed to build instrumented project")?;

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        anyhow::bail!("Failed to build instrumented project:\n{}", stderr);
    }

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

    for arg in &options.test_args {
        test_cmd.arg(arg);
    }

    let mut child = test_cmd.spawn().context("Failed to spawn go test")?;
    let mut test_output = String::new();

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                if options.verbose {
                    println!("  {}", line);
                }
                test_output.push_str(&line);
                test_output.push('\n');
            }
        }
    }

    let status = child.wait().context("Failed to wait for go test")?;
    if !status.success() && options.fail_on_error {
        anyhow::bail!("Tests failed. Enable verbose mode to see test output.");
    }

    let coverage_file = project_path.join("coverage_data.json");
    if !coverage_file.exists() {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if !coverage_file.exists() {
            anyhow::bail!(
                "Coverage data file not found. This usually means:\n\
                1. No tests were run\n\
                2. Tests crashed before coverage could be saved\n\
                3. There's an issue with the instrumentation"
            );
        }
    }

    let data = fs::read_to_string(&coverage_file).context("Failed to read coverage data")?;
    let coverage: HashMap<String, Vec<usize>> =
        serde_json::from_str(&data).context("Failed to parse coverage data")?;

    Ok((coverage, Some(test_output)))
}

fn calculate_statement_coverage(
    instrumentation_data: &InstrumentationData,
    coverage_data: &HashMap<String, Vec<usize>>,
) -> Result<ProjectCoverage> {
    let mut functions = HashMap::new();
    let mut total_statements = 0;
    let mut covered_statements = 0;

    for (func_name, total_stmts) in &instrumentation_data.total_statements_per_function {
        let mappings = instrumentation_data
            .statement_mappings
            .get(func_name)
            .ok_or_else(|| anyhow::anyhow!("No mappings found for function {}", func_name))?;

        let covered_stmt_ids: HashSet<usize> = coverage_data
            .get(func_name)
            .map(|ids| ids.iter().copied().collect())
            .unwrap_or_default();

        let covered_count = covered_stmt_ids.len();
        let mut uncovered_statements = Vec::new();
        let mut uncovered_line_map: HashMap<usize, Vec<(usize, String)>> = HashMap::new();

        for mapping in mappings {
            if !covered_stmt_ids.contains(&mapping.stmt_id) {
                uncovered_statements.push(mapping.stmt_id);
                uncovered_line_map
                    .entry(mapping.line)
                    .or_insert_with(Vec::new)
                    .push((mapping.stmt_id, mapping.stmt_type.clone()));
            }
        }

        let mut uncovered_line_details: Vec<UncoveredLine> = uncovered_line_map
            .into_iter()
            .map(|(line, stmts)| {
                let stmt_ids: Vec<usize> = stmts.iter().map(|(id, _)| *id).collect();
                let stmt_type = if stmts.len() == 1 {
                    stmts[0].1.clone()
                } else {
                    "multiple".to_string()
                };
                UncoveredLine {
                    line,
                    statement: format!("Line {}", line),
                    stmt_type,
                    stmt_ids,
                }
            })
            .collect();

        uncovered_line_details.sort_by_key(|d| d.line);
        let uncovered_lines: Vec<usize> = uncovered_line_details
            .iter()
            .map(|d| d.line)
            .collect();

        let coverage_percentage = if *total_stmts > 0 {
            (covered_count as f64 / *total_stmts as f64) * 100.0
        } else {
            100.0
        };

        functions.insert(
            func_name.clone(),
            FunctionCoverage {
                total_statements: *total_stmts,
                covered_statements: covered_count,
                coverage_percentage,
                uncovered_statements,
                uncovered_lines,
                uncovered_line_details,
            },
        );

        total_statements += total_stmts;
        covered_statements += covered_count;
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
        files_analyzed: 0,
        test_output: None,
    })
}
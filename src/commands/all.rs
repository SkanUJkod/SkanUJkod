use crate::plugin_manager::PluginManager;
use std::path::Path;
use std::fs;

pub fn run_all_analyses(
    plugin_manager: &mut PluginManager,
    project_path: &Path,
    output_dir: Option<&Path>,
    include_tests: bool,
    exclude_patterns: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running all available analyses on: {}", project_path.display());
    
    // Validate inputs
    if !project_path.exists() {
        return Err(format!("Project path '{}' does not exist", project_path.display()).into());
    }
    
    let go_mod_path = project_path.join("go.mod");
    if !go_mod_path.exists() {
        return Err(format!("No go.mod found in '{}'. Not a Go project?", project_path.display()).into());
    }
    
    // Create output directory if specified
    if let Some(dir) = output_dir {
        fs::create_dir_all(dir)?;
    }
    
    let mut results = Vec::new();
    
    // 1. CFG Analysis
    println!("\n=== Running CFG Analysis ===");
    match run_cfg_analysis(plugin_manager, project_path, output_dir, include_tests, exclude_patterns) {
        Ok(_result) => {
            results.push(("CFG Analysis", "✓ Success".to_string()));
        }
        Err(e) => {
            results.push(("CFG Analysis", format!("✗ Failed: {}", e)));
            eprintln!("CFG Analysis failed: {}", e);
        }
    }
    
    // 2. Branch Coverage Analysis
    println!("\n=== Running Branch Coverage Analysis ===");
    match run_branch_coverage_analysis(plugin_manager, project_path, output_dir, include_tests, exclude_patterns) {
        Ok(_result) => {
            results.push(("Branch Coverage", "✓ Success".to_string()));
        }
        Err(e) => {
            results.push(("Branch Coverage", format!("✗ Failed: {}", e)));
            eprintln!("Branch Coverage Analysis failed: {}", e);
        }
    }
    
    // 3. Statement Coverage Analysis
    println!("\n=== Running Statement Coverage Analysis ===");
    match run_statement_coverage_analysis(plugin_manager, project_path, output_dir, include_tests, exclude_patterns) {
        Ok(_result) => {
            results.push(("Statement Coverage", "✓ Success".to_string()));
        }
        Err(e) => {
            results.push(("Statement Coverage", format!("✗ Failed: {}", e)));
            eprintln!("Statement Coverage Analysis failed: {}", e);
        }
    }
    
    // 4. Cyclomatic Complexity Analysis
    println!("\n=== Running Cyclomatic Complexity Analysis ===");
    match run_complexity_analysis(plugin_manager, project_path, output_dir, include_tests, exclude_patterns) {
        Ok(_result) => {
            results.push(("Cyclomatic Complexity", "✓ Success".to_string()));
        }
        Err(e) => {
            results.push(("Cyclomatic Complexity", format!("✗ Failed: {}", e)));
            eprintln!("Cyclomatic Complexity Analysis failed: {}", e);
        }
    }
    
    // Generate summary report
    let summary = generate_summary_report(&results);
    
    match output_dir {
        Some(dir) => {
            let summary_path = dir.join("analysis_summary.txt");
            fs::write(&summary_path, &summary)?;
            println!("\nAll analyses completed. Summary written to: {}", summary_path.display());
        }
        None => {
            println!("\n{}", summary);
        }
    }
    
    Ok(())
}

fn run_cfg_analysis(
    plugin_manager: &mut PluginManager,
    project_path: &Path,
    output_dir: Option<&Path>,
    include_tests: bool,
    exclude_patterns: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = output_dir.map(|dir| dir.join("cfg_analysis.dot"));
    crate::commands::cfg::run_cfg_analysis(
        plugin_manager,
        project_path,
        output_path.as_deref(),
        &crate::cli::OutputFormat::Dot,
        include_tests,
        exclude_patterns,
    )
}

fn run_branch_coverage_analysis(
    plugin_manager: &mut PluginManager,
    project_path: &Path,
    output_dir: Option<&Path>,
    include_tests: bool,
    exclude_patterns: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = output_dir.map(|dir| dir.join("branch_coverage.txt"));
    crate::commands::branch_coverage::run_branch_coverage_analysis(
        plugin_manager,
        project_path,
        output_path.as_deref(),
        0.8, // Default threshold
        include_tests,
        exclude_patterns,
    )
}

fn run_statement_coverage_analysis(
    plugin_manager: &mut PluginManager,
    project_path: &Path,
    output_dir: Option<&Path>,
    include_tests: bool,
    exclude_patterns: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = output_dir.map(|dir| dir.join("statement_coverage.txt"));
    crate::commands::statement_coverage::run_statement_coverage_analysis(
        plugin_manager,
        project_path,
        output_path.as_deref(),
        0.8, // Default threshold
        include_tests,
        exclude_patterns,
    )
}

fn run_complexity_analysis(
    plugin_manager: &mut PluginManager,
    project_path: &Path,
    output_dir: Option<&Path>,
    include_tests: bool,
    exclude_patterns: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = output_dir.map(|dir| dir.join("cyclomatic_complexity.txt"));
    crate::commands::cyclomatic_complexity::run_complexity_analysis(
        plugin_manager,
        project_path,
        output_path.as_deref(),
        10, // Default max complexity
        include_tests,
        exclude_patterns,
    )
}

fn generate_summary_report(results: &[(& str, String)]) -> String {
    let mut report = String::new();
    report.push_str("SkanUJkod Analysis Summary Report\n");
    report.push_str("=================================\n\n");
    
    let mut success_count = 0;
    let total_count = results.len();
    
    for (analysis_name, result) in results {
        report.push_str(&format!("{:.<30} {}\n", analysis_name, result));
        if result.starts_with("✓") {
            success_count += 1;
        }
    }
    
    report.push_str(&format!("\nSummary: {}/{} analyses completed successfully\n", success_count, total_count));
    
    if success_count == total_count {
        report.push_str("🎉 All analyses completed successfully!\n");
    } else {
        report.push_str("⚠️  Some analyses failed. Check the error messages above.\n");
    }
    
    report
}

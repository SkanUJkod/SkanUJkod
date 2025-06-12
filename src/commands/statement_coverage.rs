use crate::plugin_manager::PluginManager;
use plugin_interface::QualPFID;
use std::path::Path;
use std::fs;

pub fn run_statement_coverage_analysis(
    plugin_manager: &mut PluginManager,
    project_path: &Path,
    output_path: Option<&Path>,
    threshold: f64,
    include_tests: bool,
    exclude_patterns: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running statement coverage analysis on: {}", project_path.display());
    
    // Validate inputs
    if !project_path.exists() {
        return Err(format!("Project path '{}' does not exist", project_path.display()).into());
    }
    
    if !(0.0..=1.0).contains(&threshold) {
        return Err("Threshold must be between 0.0 and 1.0".into());
    }
    
    let go_mod_path = project_path.join("go.mod");
    if !go_mod_path.exists() {
        return Err(format!("No go.mod found in '{}'. Not a Go project?", project_path.display()).into());
    }
    
    // Prepare user parameters
    let mut user_params = plugin_manager.create_user_params();
    plugin_manager.add_string_param(&mut user_params, "project_path", 
                                   project_path.to_string_lossy().as_ref());
    plugin_manager.add_bool_param(&mut user_params, "include_test_files", include_tests);
    plugin_manager.add_vec_param(&mut user_params, "exclude_patterns", exclude_patterns.to_vec());
    
    // Add coverage-specific parameters
    plugin_manager.add_string_param(&mut user_params, "coverage_threshold", &threshold.to_string());
    
    // Target function for statement coverage analysis
    let target_function = QualPFID {
        plugin_id: "statement_cov_plugin".into(),
        pf_id: "analyze_statement_coverage".into(),
    };
    
    // Execute plugin chain
    println!("Executing statement coverage plugin chain...");
    let result = plugin_manager.execute_plugin_chain(&target_function, user_params)?;
    
    // Format output
    let output_content = format_statement_coverage_output(&result, threshold)?;
    
    match output_path {
        Some(path) => {
            fs::write(path, &output_content)?;
            println!("Statement coverage analysis results written to: {}", path.display());
        }
        None => {
            println!("Statement Coverage Analysis Results:");
            println!("{}", output_content);
        }
    }
    
    Ok(())
}

fn format_statement_coverage_output(result: &plugin_interface::BoxedPFResult, threshold: f64) -> Result<String, Box<dyn std::error::Error>> {
    // Try to extract the coverage data from the result
    let mut output = String::new();
    
    // Header
    output.push_str("Statement Coverage Analysis Report\n");
    output.push_str("==================================\n\n");
    
    // Threshold
    output.push_str(&format!("📊 Coverage Threshold: {:.1}%\n\n", threshold * 100.0));
    
    // Try to extract structured data from the result
    // Note: This is a simplified approach since we're working with BoxedPFResult
    let result_str = format!("{:?}", result);
    
    // Parse key information from the debug output
    if let Some(start) = result_str.find("overall_coverage_percentage: ") {
        if let Some(end) = result_str[start..].find(',') {
            let coverage_str = &result_str[start + 30..start + end];
            if let Ok(coverage) = coverage_str.parse::<f64>() {
                let status_icon = if coverage >= threshold * 100.0 { "✅" } else { "❌" };
                output.push_str(&format!("📈 Overall Coverage: {:.2}% {}\n", coverage, status_icon));
            }
        }
    }
    
    // Extract total and covered statements
    if let Some(start) = result_str.find("total_statements: ") {
        if let Some(end) = result_str[start..].find(',') {
            let total_str = &result_str[start + 18..start + end];
            if let Ok(total) = total_str.parse::<usize>() {
                output.push_str(&format!("📋 Total Statements: {}\n", total));
            }
        }
    }
    
    if let Some(start) = result_str.find("covered_statements: ") {
        if let Some(end) = result_str[start..].find(',') {
            let covered_str = &result_str[start + 20..start + end];
            if let Ok(covered) = covered_str.parse::<usize>() {
                output.push_str(&format!("✅ Covered Statements: {}\n", covered));
            }
        }
    }
    
    // Extract files analyzed
    if let Some(start) = result_str.find("files_analyzed: [") {
        if let Some(end) = result_str[start..].find(']') {
            let files_section = &result_str[start + 17..start + end];
            let file_count = files_section.matches('"').count() / 2;
            output.push_str(&format!("📁 Files Analyzed: {}\n", file_count));
        }
    }
    
    // Extract uncovered statements count
    if let Some(start) = result_str.find("uncovered_statements: [") {
        let uncovered_section = &result_str[start + 23..];
        let uncovered_count = uncovered_section.matches("StatementInfo").count();
        if uncovered_count > 0 {
            output.push_str(&format!("⚠️  Uncovered Statements: {}\n", uncovered_count));
        } else {
            output.push_str("🎉 All statements covered!\n");
        }
    }
    
    output.push_str("\n");
    output.push_str("✅ Analysis completed successfully.\n");
    
    Ok(output)
}

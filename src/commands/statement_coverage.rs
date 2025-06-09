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
    // Format the statement coverage results in a human-readable way
    let output = format!(
        r#"Statement Coverage Analysis Report
==================================

Coverage Threshold: {:.1}%
Result: {:?}

Analysis completed successfully.
"#,
        threshold * 100.0,
        result
    );
    
    Ok(output)
}

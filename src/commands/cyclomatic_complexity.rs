use crate::plugin_manager::PluginManager;
use plugin_interface::QualPFID;
use std::path::Path;
use std::fs;

pub fn run_complexity_analysis(
    plugin_manager: &mut PluginManager,
    project_path: &Path,
    output_path: Option<&Path>,
    max_complexity: i32,
    include_tests: bool,
    exclude_patterns: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running cyclomatic complexity analysis on: {}", project_path.display());
    
    // Validate inputs
    if !project_path.exists() {
        return Err(format!("Project path '{}' does not exist", project_path.display()).into());
    }
    
    if max_complexity < 1 {
        return Err("Maximum complexity must be at least 1".into());
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
    
    // Add complexity-specific parameters
    plugin_manager.add_string_param(&mut user_params, "max_allowed_complexity", &max_complexity.to_string());
    
    // Target function for cyclomatic complexity analysis
    let target_function = QualPFID {
        plugin_id: "cyclomatic_complexity_plugin".into(),
        pf_id: "analyze_complexity".into(),
    };
    
    // Execute plugin chain
    println!("Executing cyclomatic complexity plugin chain...");
    let result = plugin_manager.execute_plugin_chain(&target_function, user_params)?;
    
    // Format output
    let output_content = format_complexity_output(&result, max_complexity)?;
    
    match output_path {
        Some(path) => {
            fs::write(path, &output_content)?;
            println!("Cyclomatic complexity analysis results written to: {}", path.display());
        }
        None => {
            println!("Cyclomatic Complexity Analysis Results:");
            println!("{}", output_content);
        }
    }
    
    Ok(())
}

fn format_complexity_output(result: &plugin_interface::BoxedPFResult, max_complexity: i32) -> Result<String, Box<dyn std::error::Error>> {
    // Format the complexity results in a human-readable way
    let output = format!(
        r#"Cyclomatic Complexity Analysis Report
=====================================

Maximum Allowed Complexity: {}
Result: {:?}

Complexity Levels:
- Low: 1-10
- Moderate: 11-20  
- High: 21-30
- Very High: 31+

Analysis completed successfully.
"#,
        max_complexity,
        result
    );
    
    Ok(output)
}

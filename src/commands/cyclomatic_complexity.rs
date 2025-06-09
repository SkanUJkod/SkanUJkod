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
    // Try to extract the complexity data from the result
    let mut output = String::new();
    
    // Header
    output.push_str("Cyclomatic Complexity Analysis Report\n");
    output.push_str("=====================================\n\n");
    
    // Threshold
    output.push_str(&format!("🎯 Maximum Allowed Complexity: {}\n\n", max_complexity));
    
    // Try to extract structured data from the result
    let result_str = format!("{:?}", result);
    
    // Parse key information from the debug output
    if let Some(start) = result_str.find("total_functions: ") {
        if let Some(end) = result_str[start..].find(',') {
            let total_str = &result_str[start + 17..start + end];
            if let Ok(total) = total_str.parse::<usize>() {
                output.push_str(&format!("📊 Total Functions Analyzed: {}\n", total));
            }
        }
    }
    
    // Extract average complexity
    if let Some(start) = result_str.find("average_complexity: ") {
        if let Some(end) = result_str[start..].find(',') {
            let avg_str = &result_str[start + 20..start + end];
            if let Ok(avg) = avg_str.parse::<f64>() {
                output.push_str(&format!("📈 Average Complexity: {:.2}\n", avg));
            }
        }
    }
    
    // Extract max complexity
    if let Some(start) = result_str.find("max_complexity: ") {
        if let Some(end) = result_str[start..].find(',') {
            let max_str = &result_str[start + 16..start + end];
            if let Ok(max) = max_str.parse::<usize>() {
                let status_icon = if max <= max_complexity as usize { "✅" } else { "⚠️" };
                output.push_str(&format!("🔥 Maximum Complexity: {} {}\n", max, status_icon));
            }
        }
    }
    
    // Extract max complexity function
    if let Some(start) = result_str.find("max_complexity_function: \"") {
        if let Some(end) = result_str[start + 27..].find('"') {
            let func_name = &result_str[start + 27..start + 27 + end];
            output.push_str(&format!("🎯 Most Complex Function: {}\n", func_name));
        }
    }
    
    // Extract complexity distribution
    output.push_str("\n📊 Complexity Distribution:\n");
    if let Some(start) = result_str.find("complexity_distribution:") {
        let dist_section = &result_str[start..];
        
        // Extract counts for each level
        if let Some(low_start) = dist_section.find("\"low\": ") {
            if let Some(low_end) = dist_section[low_start + 7..].find(',') {
                let low_str = &dist_section[low_start + 7..low_start + 7 + low_end];
                if let Ok(low_count) = low_str.parse::<usize>() {
                    output.push_str(&format!("   🟢 Low (1-5): {} functions\n", low_count));
                }
            }
        }
        
        if let Some(mod_start) = dist_section.find("\"moderate\": ") {
            if let Some(mod_end) = dist_section[mod_start + 12..].find(',') {
                let mod_str = &dist_section[mod_start + 12..mod_start + 12 + mod_end];
                if let Ok(mod_count) = mod_str.parse::<usize>() {
                    output.push_str(&format!("   🟡 Moderate (6-10): {} functions\n", mod_count));
                }
            }
        }
        
        if let Some(high_start) = dist_section.find("\"high\": ") {
            if let Some(high_end) = dist_section[high_start + 8..].find(',') {
                let high_str = &dist_section[high_start + 8..high_start + 8 + high_end];
                if let Ok(high_count) = high_str.parse::<usize>() {
                    output.push_str(&format!("   🟠 High (11-20): {} functions\n", high_count));
                }
            }
        }
        
        if let Some(vh_start) = dist_section.find("\"very_high\": ") {
            if let Some(vh_end) = dist_section[vh_start + 13..].find(' ') {
                let vh_str = &dist_section[vh_start + 13..vh_start + 13 + vh_end];
                if let Ok(vh_count) = vh_str.parse::<usize>() {
                    output.push_str(&format!("   🔴 Very High (21+): {} functions\n", vh_count));
                }
            }
        }
    }
    
    output.push_str("\n📚 Complexity Guidelines:\n");
    output.push_str("   • 1-5: Simple, easy to maintain\n");
    output.push_str("   • 6-10: Moderate complexity, acceptable\n");
    output.push_str("   • 11-20: High complexity, consider refactoring\n");
    output.push_str("   • 21+: Very high complexity, refactoring recommended\n");
    
    output.push_str("\n✅ Analysis completed successfully.\n");
    
    Ok(output)
}

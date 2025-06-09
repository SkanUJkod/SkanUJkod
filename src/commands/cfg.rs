use crate::plugin_manager::PluginManager;
use crate::cli::OutputFormat;
use plugin_interface::QualPFID;
use std::path::Path;
use std::fs;

pub fn run_cfg_analysis(
    plugin_manager: &mut PluginManager,
    project_path: &Path,
    output_path: Option<&Path>,
    format: &OutputFormat,
    include_tests: bool,
    exclude_patterns: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running CFG analysis on: {}", project_path.display());
    
    // Validate project path
    if !project_path.exists() {
        return Err(format!("Project path '{}' does not exist", project_path.display()).into());
    }
    
    // Check if it's a Go project
    let go_mod_path = project_path.join("go.mod");
    if !go_mod_path.exists() {
        return Err(format!("No go.mod found in '{}'. Not a Go project?", project_path.display()).into());
    }
    
    // Prepare user parameters
    let user_params = plugin_manager.create_user_params();
    // Don't add any parameters for now to test if the issue is with parameter passing
    // plugin_manager.add_string_param(&mut user_params, "project_path", 
    //                                project_path.to_string_lossy().as_ref());
    // plugin_manager.add_bool_param(&mut user_params, "include_test_files", include_tests);
    // plugin_manager.add_vec_param(&mut user_params, "exclude_patterns", exclude_patterns.to_vec());
    
    // Define target function based on desired output format
    let target_function = match format {
        OutputFormat::Dot => QualPFID {
            plugin_id: "cfg_plugin".into(),
            pf_id: "export_dot".into(),
        },
        OutputFormat::Json => QualPFID {
            plugin_id: "cfg_plugin".into(),
            pf_id: "build_cfg".into(), // We'll need to add JSON export
        },
        OutputFormat::Text => QualPFID {
            plugin_id: "cfg_plugin".into(),
            pf_id: "build_cfg".into(), // We'll need to add text export
        },
    };
    
    // Execute plugin chain
    println!("Executing CFG plugin chain...");
    let result = plugin_manager.execute_plugin_chain(&target_function, user_params)?;
    
    // Handle output
    let output_content = format_cfg_output(&result, format)?;
    
    match output_path {
        Some(path) => {
            fs::write(path, &output_content)?;
            println!("CFG analysis results written to: {}", path.display());
        }
        None => {
            println!("CFG Analysis Results:");
            println!("{}", output_content);
        }
    }
    
    Ok(())
}

fn format_cfg_output(result: &plugin_interface::BoxedPFResult, format: &OutputFormat) -> Result<String, Box<dyn std::error::Error>> {
    // For now, we'll extract the result as a string
    // In a real implementation, you'd need to properly deserialize the result
    // based on the plugin's return type
    
    match format {
        OutputFormat::Dot => {
            // Assume the result contains DOT format data
            Ok(format!("// CFG in DOT format\n// Result: {:?}", result))
        }
        OutputFormat::Json => {
            // Convert to JSON representation
            Ok(format!(r#"{{"cfg_analysis": "Result: {:?}"}}"#, result))
        }
        OutputFormat::Text => {
            // Human-readable text format
            Ok(format!("Control Flow Graph Analysis\n============================\n\nResult: {:?}", result))
        }
    }
}

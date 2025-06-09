use crate::plugin_manager::PluginManager;
use crate::cli::OutputFormat;
use crate::ui::UI;
use plugin_interface::QualPFID;
use std::path::Path;
use std::fs;

pub fn run_cfg_analysis(
    plugin_manager: &mut PluginManager,
    project_path: &Path,
    output_path: Option<&Path>,
    format: &OutputFormat,
    function_filter: Option<&str>,
    _include_tests: bool,
    _exclude_patterns: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    // Print analysis summary
    UI::print_analysis_summary("Control Flow Graph", &project_path.display().to_string(), function_filter);
    
    // Validate project path
    UI::print_subsection("Validating project");
    if !project_path.exists() {
        return Err(format!("Project path '{}' does not exist", project_path.display()).into());
    }
    UI::print_success("Project path exists");
    
    // Check if it's a Go project
    let go_mod_path = project_path.join("go.mod");
    if !go_mod_path.exists() {
        return Err(format!("No go.mod found in '{}'. Not a Go project?", project_path.display()).into());
    }
    UI::print_success("Go project detected (go.mod found)");
    
    // Prepare user parameters
    UI::print_subsection("Preparing analysis parameters");
    let mut user_params = plugin_manager.create_user_params();
    
    // Add function filter if specified
    if let Some(func) = function_filter {
        plugin_manager.add_string_param(&mut user_params, "function_filter", func);
        UI::print_kv("Function filter", func);
    }
    
    // Add output path for DOT export
    if let Some(output) = output_path {
        plugin_manager.add_string_param(&mut user_params, "output_path", output.to_string_lossy().as_ref());
        UI::print_kv("Output file", &output.display().to_string());
    }
    
    // Don't add other parameters for now to avoid ABI issues
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
    UI::print_subsection("Executing analysis");
    let result = plugin_manager.execute_plugin_chain(&target_function, user_params)?;
    
    // Handle output
    UI::print_results_header(&format!("{:?}", format));
    let output_content = format_cfg_output(&result, format)?;
    
    match output_path {
        Some(path) => {
            fs::write(path, &output_content)?;
            UI::print_file_output(&path.display().to_string());
        }
        None => {
            UI::print_subsection("Results");
            println!("{}", output_content);
        }
    }
    
    Ok(())
}

fn format_cfg_output(result: &plugin_interface::BoxedPFResult, format: &OutputFormat) -> Result<String, Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Dot => {
            // For DOT format, the plugin writes directly to output.dot file
            // We need to read the actual file content instead of just showing metadata
            match fs::read_to_string("output.dot") {
                Ok(content) => Ok(content),
                Err(_) => {
                    // Fallback to showing result metadata if file reading fails
                    Ok(format!("// CFG in DOT format\n// Result: {:?}", result))
                }
            }
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

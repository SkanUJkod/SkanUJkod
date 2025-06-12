use crate::plugin_manager::PluginManager;
use crate::ui::UI;

pub fn list_available_plugins(plugin_manager: &PluginManager) -> Result<(), Box<dyn std::error::Error>> {
    UI::print_plugins_header();
    
    let connectors = plugin_manager.list_plugins();
    
    if connectors.is_empty() {
        UI::print_warning("No plugins found!");
        UI::print_info("Make sure PLUGINS_DIR environment variable points to a directory containing plugin files.");
        return Ok(());
    }
    
    // Group by plugin
    let mut plugins: std::collections::BTreeMap<String, Vec<&plugin_interface::PFConnector>> = 
        std::collections::BTreeMap::new();
    
    for connector in &connectors {
        let plugin_id = connector.pf_id.plugin_id.as_str();
        plugins.entry(plugin_id.to_string()).or_default().push(connector);
    }
    
    for (plugin_id, functions) in &plugins {
        UI::print_plugin_info(plugin_id, functions.len());
        
        for function in functions {
            let dependencies: Vec<String> = function.pf_type.pf_dependencies.iter()
                .map(|dep| format!("{}::{}", dep.plugin_id, dep.pf_id))
                .collect();
                
            let params: Vec<String> = function.pf_type.user_params.iter()
                .map(|p| p.to_string())
                .collect();
                
            UI::print_function_details(&function.pf_id.pf_id, &dependencies, &params);
        }
    }
    
    println!();
    UI::print_kv("Total plugins", &plugins.len().to_string());
    UI::print_kv("Total functions", &connectors.len().to_string());
    
    Ok(())
}

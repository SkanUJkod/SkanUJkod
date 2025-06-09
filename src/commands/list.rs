use crate::plugin_manager::PluginManager;

pub fn list_available_plugins(plugin_manager: &PluginManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("Available Plugins and Functions");
    println!("===============================\n");
    
    let connectors = plugin_manager.list_plugins();
    
    if connectors.is_empty() {
        println!("No plugins found!");
        println!("Make sure PLUGINS_DIR environment variable points to a directory containing plugin files.");
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
        println!("🔌 Plugin: {}", plugin_id);
        println!("   Functions:");
        
        for function in functions {
            println!("   • {}::{}", 
                   function.pf_id.plugin_id, 
                   function.pf_id.pf_id);
            
            // Show dependencies
            if !function.pf_type.pf_dependencies.is_empty() {
                println!("     Dependencies:");
                for dep in &function.pf_type.pf_dependencies {
                    println!("     - {}::{}", dep.plugin_id, dep.pf_id);
                }
            }
            
            // Show required user parameters
            if !function.pf_type.user_params.is_empty() {
                println!("     Required Parameters:");
                for param in &function.pf_type.user_params {
                    println!("     - {}", param);
                }
            }
            
            println!();
        }
        println!();
    }
    
    println!("Total: {} plugins with {} functions", plugins.len(), connectors.len());
    
    Ok(())
}

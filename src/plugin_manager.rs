use abi_stable::{
    DynTrait, library::lib_header_from_path, std_types::{RHashMap, RString, ROption, Tuple2}
};
use plugin_interface::{
    BoxedPFResult, BoxedUserParam, PFConnector, PFDependencies, PluginRef, QualPFID, UserParameters
};
use crate::ui::UI;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

pub struct PluginManager {
    connectors: Vec<PFConnector>,
    pf_to_idx: HashMap<QualPFID, usize>,
}

impl PluginManager {
    pub fn new(plugins_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let plugins_path = Path::new(plugins_dir);
        
        if !plugins_path.exists() {
            return Err(format!("Plugins directory '{}' does not exist", plugins_dir).into());
        }

        // Find all plugin files (*.dylib on macOS, *.so on Linux, *.dll on Windows)
        let plugin_files = Self::find_plugin_files(plugins_path)?;
        
        if plugin_files.is_empty() {
            return Err(format!("No plugin files found in '{}'", plugins_dir).into());
        }

        // Load all plugins
        let connectors = Self::load_plugins(plugin_files)?;
        
        if connectors.is_empty() {
            return Err("No plugin functions found".into());
        }

        let pf_to_idx: HashMap<QualPFID, usize> = connectors
            .iter()
            .enumerate()
            .map(|(i, c)| (c.pf_id.clone(), i))
            .collect();

        Ok(PluginManager {
            connectors,
            pf_to_idx,
        })
    }

    fn find_plugin_files(plugins_dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut plugin_files = Vec::new();
        
        for entry in fs::read_dir(plugins_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext_str = extension.to_string_lossy();
                    // Check for dynamic library extensions
                    if ext_str == "dylib" || ext_str == "so" || ext_str == "dll" {
                        // Filter for our plugin naming convention
                        if let Some(name) = path.file_stem() {
                            let name_str = name.to_string_lossy();
                            if name_str.starts_with("lib") && 
                               (name_str.contains("_plugin") || name_str.contains("plugin")) {
                                plugin_files.push(path);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(plugin_files)
    }

    fn load_plugins(plugin_files: Vec<PathBuf>) -> Result<Vec<PFConnector>, Box<dyn std::error::Error>> {
        let mut all_connectors = Vec::new();
        
        for plugin_path in plugin_files {
            println!("Loading plugin: {}", plugin_path.display());
            
            match lib_header_from_path(&plugin_path) {
                Ok(header) => {
                    match header.init_root_module::<PluginRef>() {
                        Ok(lib) => {
                            let connectors = lib.funcs()();
                            println!("  Loaded {} functions from plugin", connectors.len());
                            all_connectors.extend(connectors.into_iter());
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to initialize plugin {}: {}", 
                                    plugin_path.display(), e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load plugin {}: {}", 
                            plugin_path.display(), e);
                }
            }
        }
        
        Ok(all_connectors)
    }

    pub fn execute_plugin_chain(&mut self, 
                               target_function: &QualPFID, 
                               user_params: UserParameters) -> Result<BoxedPFResult, Box<dyn std::error::Error>> {
        // Build dependency graph and topological order
        let topo_order = self.build_execution_order(target_function)?;
        
        // Execute plugins in topological order
        let mut results: RHashMap<QualPFID, BoxedPFResult> = RHashMap::new();
        
        for &idx in &topo_order {
            let connector = &self.connectors[idx];
            
            UI::print_plugin_execution(&connector.pf_id.plugin_id, &connector.pf_id.pf_id);
            
            // Prepare dependencies
            let dependencies = results
                .iter_mut()
                .filter(|Tuple2(k, _)| connector.pf_type.pf_dependencies.contains(k))
                .map(|Tuple2(k, v)| Tuple2(k.clone(), v))
                .collect::<PFDependencies>();
            
            // Execute plugin function
            let result = (connector.pf.0)(dependencies, &user_params);
            results.insert(connector.pf_id.clone(), result);
        }
        
        // Return the result of the target function
        match results.remove(target_function) {
            ROption::RSome(rresult) => Ok(rresult),
            ROption::RNone => Err(format!("Target function {}::{} was not executed", 
                               target_function.plugin_id, 
                               target_function.pf_id).into())
        }
    }

    fn build_execution_order(&self, target_function: &QualPFID) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
        let n = self.connectors.len();
        let mut adj_list: Vec<Vec<usize>> = vec![vec![]; n];
        
        // Build adjacency list for dependencies
        for (i, connector) in self.connectors.iter().enumerate() {
            for dep_id in &connector.pf_type.pf_dependencies {
                if let Some(&dep_idx) = self.pf_to_idx.get(dep_id) {
                    adj_list[dep_idx].push(i);
                } else {
                    return Err(format!("Dependency not found: {}::{}", 
                                     dep_id.plugin_id, dep_id.pf_id).into());
                }
            }
        }
        
        // Find functions needed for target
        let target_idx = self.pf_to_idx.get(target_function)
            .ok_or_else(|| format!("Target function not found: {}::{}", 
                                 target_function.plugin_id, 
                                 target_function.pf_id))?;
        
        let needed_functions = self.find_needed_functions(*target_idx, &adj_list);
        
        // Topological sort of needed functions
        let mut vis = vec![false; n];
        let mut topo_order = Vec::new();
        
        for &idx in &needed_functions {
            if !vis[idx] {
                self.dfs(idx, &adj_list, &mut vis, &mut topo_order, &needed_functions);
            }
        }
        
        topo_order.reverse();
        Ok(topo_order)
    }
    
    fn find_needed_functions(&self, target_idx: usize, _adj_list: &[Vec<usize>]) -> Vec<usize> {
        let mut needed = Vec::new();
        let mut visited = vec![false; self.connectors.len()];
        self.collect_dependencies(target_idx, &mut visited, &mut needed);
        needed
    }
    
    fn collect_dependencies(&self, idx: usize, visited: &mut Vec<bool>, needed: &mut Vec<usize>) {
        if visited[idx] {
            return;
        }
        visited[idx] = true;
        needed.push(idx);
        
        for dep_id in &self.connectors[idx].pf_type.pf_dependencies {
            if let Some(&dep_idx) = self.pf_to_idx.get(dep_id) {
                self.collect_dependencies(dep_idx, visited, needed);
            }
        }
    }
    
    fn dfs(&self, v: usize, adj_list: &[Vec<usize>], vis: &mut Vec<bool>, 
           toposort: &mut Vec<usize>, needed: &[usize]) {
        vis[v] = true;
        for &u in &adj_list[v] {
            if needed.contains(&u) && !vis[u] {
                self.dfs(u, adj_list, vis, toposort, needed);
            }
        }
        toposort.push(v);
    }

    pub fn list_plugins(&self) -> Vec<&PFConnector> {
        self.connectors.iter().collect()
    }

    pub fn create_user_params(&self) -> UserParameters {
        RHashMap::new()
    }

    pub fn add_string_param(&self, params: &mut UserParameters, key: &str, value: &str) {
        println!("Adding string param: {} = {}", key, value);
        let rstring_value: RString = value.into();
        let boxed_value: BoxedUserParam = DynTrait::from_value(rstring_value);
        params.insert(key.into(), boxed_value);
        println!("String param added successfully");
    }

    pub fn add_bool_param(&self, params: &mut UserParameters, key: &str, value: bool) {
        let boxed_value: BoxedUserParam = DynTrait::from_value(value);
        params.insert(key.into(), boxed_value);
    }

    pub fn add_vec_param(&self, params: &mut UserParameters, key: &str, value: Vec<String>) {
        // For now, we'll serialize the vector as a JSON string
        // This is a simplified approach - in production you might want a more sophisticated method
        let json_value = serde_json::to_string(&value).unwrap_or_default();
        let rstring_value: RString = json_value.into();
        let boxed_value: BoxedUserParam = DynTrait::from_value(rstring_value);
        params.insert(key.into(), boxed_value);
    }
}

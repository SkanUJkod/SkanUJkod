use abi_stable::{rvec, std_types::RVec};
use plugin_interface::{
    BoxedPFResult, PFConnector, PFDependencies, PFType, Plugin, PluginRef,
    PluginFunction, QualPFID, UserParameters,
};
use std::fmt::{self, Display};

use abi_stable::{
    DynTrait, export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn, std_types::RString,
};

// Re-export the core CFG functionality
pub mod ast;
pub mod cfg;
pub mod export;

pub use ast::{ParseOptions, ParsedFile, parse_project, parse_project_with_options};
pub use cfg::{BasicBlock, ControlFlowGraph, build_cfgs_for_file};
pub use export::to_dot;

#[export_root_module]
pub fn get_library() -> PluginRef {
    Plugin { funcs: new_pf_vec }.leak_into_prefix()
}

#[sabi_extern_fn]
fn new_pf_vec() -> RVec<PFConnector> {
    rvec![
        // Plugin function to parse a Go project
        PFConnector {
            pf: PluginFunction(parse_project_pf),
            pf_type: PFType {
                pf_dependencies: rvec![],
                user_params: rvec!["project_path".into(), "include_test_files".into(), "exclude_patterns".into()]
            },
            pf_id: QualPFID {
                plugin_id: "cfg_plugin".into(),
                pf_id: "parse_project".into()
            }
        },
        // Plugin function to build CFG from parsed project
        PFConnector {
            pf: PluginFunction(build_cfg_pf),
            pf_type: PFType {
                pf_dependencies: rvec![QualPFID {
                    plugin_id: "cfg_plugin".into(),
                    pf_id: "parse_project".into()
                }],
                user_params: rvec![]
            },
            pf_id: QualPFID {
                plugin_id: "cfg_plugin".into(),
                pf_id: "build_cfg".into()
            }
        },
        // Plugin function to export CFG to DOT format
        PFConnector {
            pf: PluginFunction(export_dot_pf),
            pf_type: PFType {
                pf_dependencies: rvec![QualPFID {
                    plugin_id: "cfg_plugin".into(),
                    pf_id: "build_cfg".into()
                }],
                user_params: rvec!["output_path".into()]
            },
            pf_id: QualPFID {
                plugin_id: "cfg_plugin".into(),
                pf_id: "export_dot".into()
            }
        }
    ]
}

// Result types for plugin functions
#[derive(Debug)]
pub struct ParsedProjectResult {
    pub file_set: go_parser::FileSet,
    pub ast_objects: go_parser::AstObjects,
    pub files: Vec<ParsedFile>,
}

impl Display for ParsedProjectResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParsedProject with {} files", self.files.len())
    }
}

#[derive(Debug)]
pub struct CFGResult {
    pub cfgs: Vec<(String, Vec<ControlFlowGraph>)>, // (filename, cfgs)
}

impl Display for CFGResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CFG for {} files", self.cfgs.len())
    }
}

#[derive(Debug)]
pub struct ExportResult {
    pub output_path: String,
    pub success: bool,
}

impl Display for ExportResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Export to {} - success: {}", self.output_path, self.success)
    }
}

// Plugin function implementations
#[sabi_extern_fn]
fn parse_project_pf(_pf_results: PFDependencies, user_params: &UserParameters) -> BoxedPFResult<'static> {
    println!("parse_project_pf called!");
    println!("user_params size: {}", user_params.len());
    
    // Debug: print all available keys
    for key in user_params.keys() {
        println!("Available parameter key: {}", key);
    }
    
    // For now, let's use a hardcoded path to test if the issue is with parameter parsing
    let project_path = "../go-code/example-go";
    println!("Using hardcoded project path: {}", project_path);
    
    let include_test_files = false; // Hardcoded for now due to ABI issue
    println!("Using hardcoded include_test_files: {}", include_test_files);
    
    let exclude_patterns: Vec<String> = Vec::new(); // Hardcoded for now due to ABI issue
    println!("Using hardcoded exclude_patterns: {:?}", exclude_patterns);

    let options = ParseOptions {
        exclude_patterns,
        include_test_files,
    };

    match parse_project_with_options(std::path::Path::new(project_path), &options) {
        Ok((file_set, ast_objects, files)) => {
            DynTrait::from_value(ParsedProjectResult {
                file_set,
                ast_objects,
                files,
            })
        }
        Err(e) => {
            eprintln!("Error parsing project: {}", e);
            DynTrait::from_value(ParsedProjectResult {
                file_set: go_parser::FileSet::new(),
                ast_objects: go_parser::AstObjects::new(),
                files: Vec::new(),
            })
        }
    }
}

#[sabi_extern_fn]
fn build_cfg_pf(pf_results: PFDependencies, _user_params: &UserParameters) -> BoxedPFResult<'static> {
    // Get parsed project from dependencies
    let parsed_project_id = QualPFID {
        plugin_id: "cfg_plugin".into(),
        pf_id: "parse_project".into(),
    };
    
    let parsed_result = pf_results.get(&parsed_project_id)
        .expect("Missing parsed project dependency");
    
    let parsed_project = unsafe { 
        parsed_result.unchecked_downcast_as::<ParsedProjectResult>() 
    };

    let mut cfgs = Vec::new();
    
    for parsed_file in &parsed_project.files {
        let file_cfgs = build_cfgs_for_file(&parsed_project.file_set, &parsed_project.ast_objects, &parsed_file.ast);
        // Convert HashMap to Vec of (function_name, cfg) tuples to preserve function names
        let cfg_vec: Vec<(String, ControlFlowGraph)> = file_cfgs.into_iter().collect();
        for (func_name, cfg) in cfg_vec {
            cfgs.push((
                format!("{}::{}", parsed_file.path.to_string_lossy(), func_name),
                vec![cfg]
            ));
        }
    }

    DynTrait::from_value(CFGResult { cfgs })
}

#[sabi_extern_fn]
fn export_dot_pf(pf_results: PFDependencies, user_params: &UserParameters) -> BoxedPFResult<'static> {
    // Get CFG result from dependencies
    let cfg_result_id = QualPFID {
        plugin_id: "cfg_plugin".into(),
        pf_id: "build_cfg".into(),
    };
    
    let cfg_result = pf_results.get(&cfg_result_id)
        .expect("Missing CFG dependency");
    
    let cfg_data = unsafe { 
        cfg_result.unchecked_downcast_as::<CFGResult>() 
    };

    let output_path = user_params.get("output_path")
        .map(|p| unsafe { p.unchecked_downcast_as::<RString>() }.as_str())
        .unwrap_or("output.dot");

    // Create DOT output for all CFGs
    let mut dot_content = String::from("digraph G {\n");
    
    for (file_and_func, cfgs) in &cfg_data.cfgs {
        for (i, cfg) in cfgs.iter().enumerate() {
            // Extract function name from "filename::function_name" format
            let display_name = if file_and_func.contains("::") {
                file_and_func.clone()
            } else {
                format!("{}::{}", file_and_func, i)
            };
            
            // Create safe identifier for DOT subgraph
            let safe_id = crate::export::sanitize_function_name(&file_and_func.replace("::", "_"));
            
            dot_content.push_str(&format!("  subgraph cluster_{}_{} {{\n", safe_id, i));
            dot_content.push_str(&format!("    label=\"{}\";\n", display_name.replace('\"', "\\\"")));
            dot_content.push_str(&to_dot(cfg));
            dot_content.push_str("  }\n");
        }
    }
    
    dot_content.push_str("}\n");

    let success = match std::fs::write(output_path, dot_content) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Error writing DOT file: {}", e);
            false
        }
    };

    DynTrait::from_value(ExportResult {
        output_path: output_path.to_string(),
        success,
    })
}

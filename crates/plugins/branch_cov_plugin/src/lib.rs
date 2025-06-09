use abi_stable::{rvec, std_types::RVec};
use plugin_interface::{
    BoxedPFResult, PFConnector, PFDependencies, PFType, Plugin, PluginRef,
    PluginFunction, QualPFID, UserParameters,
};
use std::fmt::{self, Display};

use abi_stable::{
    DynTrait, export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn, std_types::RString,
};

// Re-export the core branch coverage functionality
pub mod core;
pub mod helpers;
pub mod instrumentation;

pub use core::{
    analyze_branch_coverage, analyze_branch_coverage_with_options, BranchCoverageOptions,
    BranchInfo, FunctionBranchCoverage, ProjectBranchCoverage, UncoveredBranch,
};

#[export_root_module]
pub fn get_library() -> PluginRef {
    Plugin { funcs: new_pf_vec }.leak_into_prefix()
}

#[sabi_extern_fn]
fn new_pf_vec() -> RVec<PFConnector> {
    rvec![
        // Plugin function to analyze branch coverage
        PFConnector {
            pf: PluginFunction(analyze_branch_coverage_pf),
            pf_type: PFType {
                pf_dependencies: rvec![QualPFID {
                    plugin_id: "cfg_plugin".into(),
                    pf_id: "build_cfg".into()
                }],
                user_params: rvec![
                    "project_path".into(),
                    "verbose".into(),
                    "min_coverage_threshold".into(),
                    "exclude_patterns".into(),
                    "test_args".into(),
                    "timeout_seconds".into()
                ]
            },
            pf_id: QualPFID {
                plugin_id: "branch_cov_plugin".into(),
                pf_id: "analyze_branch_coverage".into()
            }
        },
        // Plugin function to generate branch coverage report
        PFConnector {
            pf: PluginFunction(generate_report_pf),
            pf_type: PFType {
                pf_dependencies: rvec![QualPFID {
                    plugin_id: "branch_cov_plugin".into(),
                    pf_id: "analyze_branch_coverage".into()
                }],
                user_params: rvec!["output_format".into(), "output_path".into()]
            },
            pf_id: QualPFID {
                plugin_id: "branch_cov_plugin".into(),
                pf_id: "generate_report".into()
            }
        }
    ]
}

// Result types for plugin functions
#[derive(Debug)]
pub struct BranchCoverageResult {
    pub project_coverage: ProjectBranchCoverage,
}

impl Display for BranchCoverageResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Branch Coverage: {:.2}% ({}/{} branches covered)", 
               self.project_coverage.overall_coverage_percentage,
               self.project_coverage.covered_branches,
               self.project_coverage.total_branches)
    }
}

#[derive(Debug)]
pub struct ReportResult {
    pub output_path: String,
    pub success: bool,
    pub format: String,
}

impl Display for ReportResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Report generated: {} ({}), success: {}", 
               self.output_path, self.format, self.success)
    }
}

// Plugin function implementations
#[sabi_extern_fn]
fn analyze_branch_coverage_pf(_pf_results: PFDependencies, user_params: &UserParameters) -> BoxedPFResult<'static> {
    // Extract user parameters
    let project_path = user_params.get("project_path")
        .map(|p| unsafe { p.unchecked_downcast_as::<RString>() }.as_str())
        .unwrap_or(".");

    let verbose = user_params.get("verbose")
        .map(|p| unsafe { p.unchecked_downcast_as::<RString>() }.as_str() == "true")
        .unwrap_or(false);

    let min_coverage_threshold = user_params.get("min_coverage_threshold")
        .and_then(|p| unsafe { p.unchecked_downcast_as::<RString>() }.as_str().parse::<f64>().ok())
        .unwrap_or(80.0);

    let exclude_patterns = user_params.get("exclude_patterns")
        .map(|p| {
            let patterns_str = unsafe { p.unchecked_downcast_as::<RString>() }.as_str();
            if patterns_str.is_empty() {
                vec!["*_test.go".to_string(), "vendor/*".to_string()]
            } else {
                patterns_str.split(',').map(|s| s.trim().to_string()).collect()
            }
        })
        .unwrap_or_else(|| vec!["*_test.go".to_string(), "vendor/*".to_string()]);

    let test_args = user_params.get("test_args")
        .map(|p| {
            let args_str = unsafe { p.unchecked_downcast_as::<RString>() }.as_str();
            if args_str.is_empty() {
                Vec::new()
            } else {
                args_str.split_whitespace().map(|s| s.to_string()).collect()
            }
        })
        .unwrap_or_default();

    let timeout_seconds = user_params.get("timeout_seconds")
        .and_then(|p| unsafe { p.unchecked_downcast_as::<RString>() }.as_str().parse::<u64>().ok())
        .unwrap_or(30);

    let options = BranchCoverageOptions {
        verbose,
        include_test_files: false,
        min_coverage_threshold,
        fail_on_low_coverage: false,
        exclude_patterns,
        simulate_coverage: false,
        test_args,
        fail_on_error: false,
        timeout_seconds,
    };

    match analyze_branch_coverage_with_options(std::path::Path::new(project_path), &options) {
        Ok(project_coverage) => {
            DynTrait::from_value(BranchCoverageResult {
                project_coverage,
            })
        }
        Err(e) => {
            eprintln!("Error analyzing branch coverage: {}", e);
            DynTrait::from_value(BranchCoverageResult {
                project_coverage: ProjectBranchCoverage {
                    functions: std::collections::HashMap::new(),
                    total_branches: 0,
                    covered_branches: 0,
                    overall_coverage_percentage: 0.0,
                    files_analyzed: Vec::new(),
                    uncovered_branches: Vec::new(),
                    test_output: None,
                },
            })
        }
    }
}

#[sabi_extern_fn]
fn generate_report_pf(pf_results: PFDependencies, user_params: &UserParameters) -> BoxedPFResult<'static> {
    // Get branch coverage result from dependencies
    let coverage_result_id = QualPFID {
        plugin_id: "branch_cov_plugin".into(),
        pf_id: "analyze_branch_coverage".into(),
    };
    
    let coverage_result = pf_results.get(&coverage_result_id)
        .expect("Missing branch coverage dependency");
    
    let coverage_data = unsafe { 
        coverage_result.unchecked_downcast_as::<BranchCoverageResult>() 
    };

    let output_format = user_params.get("output_format")
        .map(|p| unsafe { p.unchecked_downcast_as::<RString>() }.as_str())
        .unwrap_or("json");

    let output_path = user_params.get("output_path")
        .map(|p| unsafe { p.unchecked_downcast_as::<RString>() }.as_str())
        .unwrap_or("branch_coverage_report");

    let full_output_path = format!("{}.{}", output_path, 
        if output_format == "json" { "json" } else { "txt" });

    let success = match output_format {
        "json" => {
            match serde_json::to_string_pretty(&coverage_data.project_coverage) {
                Ok(json_content) => {
                    std::fs::write(&full_output_path, json_content).is_ok()
                }
                Err(e) => {
                    eprintln!("Error serializing to JSON: {}", e);
                    false
                }
            }
        }
        _ => {
            let text_content = format!(
                "Branch Coverage Report\n\
                =====================\n\
                Overall Coverage: {:.2}%\n\
                Total Branches: {}\n\
                Covered Branches: {}\n\
                Files Analyzed: {}\n\n\
                Uncovered Branches:\n\
                {}\n",
                coverage_data.project_coverage.overall_coverage_percentage,
                coverage_data.project_coverage.total_branches,
                coverage_data.project_coverage.covered_branches,
                coverage_data.project_coverage.files_analyzed.len(),
                coverage_data.project_coverage.uncovered_branches
                    .iter()
                    .map(|b| format!("  - {}:{} ({})", b.file_path, b.line, b.branch_type))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            std::fs::write(&full_output_path, text_content).is_ok()
        }
    };

    DynTrait::from_value(ReportResult {
        output_path: full_output_path,
        success,
        format: output_format.to_string(),
    })
}

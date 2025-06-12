use abi_stable::{
    export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn, 
    std_types::{RString, RVec}, rvec, DynTrait,
};
use plugin_interface::{
    BoxedPFResult, PFDependencies, UserParameters, PluginFunction, Plugin, PluginRef,
    PFConnector, PFType, QualPFID,
};
use serde::{Deserialize, Serialize};
use std::{fmt, collections::HashMap};
use go_parser::ast::Stmt;

// Plugin metadata
const PLUGIN_NAME: &str = "cyclomatic_complexity_plugin";

// Real complexity analysis structures (based on original implementation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityAnalysisResult {
    pub functions: HashMap<String, FunctionComplexity>,
    pub total_functions: usize,
    pub average_complexity: f64,
    pub max_complexity: usize,
    pub max_complexity_function: String,
    pub complexity_distribution: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionComplexity {
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
    pub lines_of_code: usize,
    pub decision_points: Vec<DecisionPoint>,
    pub complexity_level: ComplexityLevel,
    pub nesting_depth_max: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPoint {
    pub line: usize,
    pub stmt_type: String,
    pub nesting_level: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Low,
    Moderate,
    High,
    VeryHigh,
}

impl fmt::Display for ComplexityAnalysisResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Complexity Analysis: {} functions, avg: {:.1}, max: {} ({})",
            self.total_functions, self.average_complexity, self.max_complexity, self.max_complexity_function
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityReportResult {
    pub report_path: RString,
    pub format: RString,
    pub summary: ComplexityAnalysisResult,
}

impl fmt::Display for ComplexityReportResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Complexity report generated: {}", self.report_path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportResult {
    pub report_path: String,
    pub format: String,
}

impl fmt::Display for ReportResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Report generated: {} ({})", self.report_path, self.format)
    }
}

// CFG type definitions compatible with cfg_plugin
#[derive(Debug, Clone)]
pub struct Statement {
    pub text: String,
    pub stmt: go_parser::ast::Stmt,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,
    pub stmts: Vec<Statement>,
    pub succs: Vec<usize>,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub blocks: HashMap<usize, BasicBlock>,
    pub entry: usize,
}

#[derive(Debug)]
pub struct CFGResult {
    pub cfgs: Vec<(String, Vec<ControlFlowGraph>)>, // (filename, cfgs)
}

// Export the plugin
#[export_root_module]
pub fn get_library() -> PluginRef {
    Plugin { funcs: new_pf_vec }.leak_into_prefix()
}

#[sabi_extern_fn]
fn new_pf_vec() -> RVec<PFConnector> {
    rvec![
        PFConnector {
            pf: PluginFunction(analyze_complexity_pf),
            pf_type: PFType {
                pf_dependencies: rvec![QualPFID {
                    plugin_id: "cfg_plugin".into(),
                    pf_id: "build_cfg".into(),
                }],
                user_params: rvec!["include_cognitive".into(), "max_allowed_complexity".into()],
            },
            pf_id: QualPFID {
                plugin_id: PLUGIN_NAME.into(),
                pf_id: "analyze_complexity".into(),
            },
        },
        PFConnector {
            pf: PluginFunction(generate_report_pf),
            pf_type: PFType {
                pf_dependencies: rvec![QualPFID {
                    plugin_id: PLUGIN_NAME.into(),
                    pf_id: "analyze_complexity".into(),
                }],
                user_params: rvec!["output_path".into(), "format".into()],
            },
            pf_id: QualPFID {
                plugin_id: PLUGIN_NAME.into(),
                pf_id: "generate_report".into(),
            },
        }
    ]
}

#[sabi_extern_fn]
fn analyze_complexity_pf(pf_results: PFDependencies, user_params: &UserParameters) -> BoxedPFResult<'static> {
    // Get CFG data from cfg_plugin
    let cfg_key = QualPFID {
        plugin_id: "cfg_plugin".into(),
        pf_id: "build_cfg".into(),
    };
    
    let cfg_result = pf_results.get(&cfg_key)
        .expect("Missing CFG analysis result");
    
    let cfg_data = unsafe { 
        cfg_result.unchecked_downcast_as::<CFGResult>() 
    };

    // Get user parameters
    let include_cognitive = user_params.get("include_cognitive")
        .map(|p| unsafe { p.unchecked_downcast_as::<RString>() }.as_str())
        .unwrap_or("true") == "true";

    let _max_allowed_complexity = user_params.get("max_allowed_complexity")
        .map(|p| unsafe { p.unchecked_downcast_as::<RString>() }.as_str())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);

    // Analyze complexity for each function using real algorithm
    let mut functions = HashMap::new();
    let mut max_complexity = 0;
    let mut max_complexity_function = String::new();

    for (filename, file_cfgs) in &cfg_data.cfgs {
        for cfg in file_cfgs {
            // Use a combination of filename and entry block as function identifier
            let func_name = format!("{}::{}", filename, cfg.entry);
            let func_complexity = analyze_function_complexity(cfg, include_cognitive);

            if func_complexity.cyclomatic_complexity > max_complexity {
                max_complexity = func_complexity.cyclomatic_complexity;
                max_complexity_function = func_name.clone();
            }

            functions.insert(func_name, func_complexity);
        }
    }

    // Calculate statistics
    let total_functions = functions.len();
    let total_complexity: usize = functions.values().map(|f| f.cyclomatic_complexity).sum();
    let average_complexity = if total_functions > 0 {
        total_complexity as f64 / total_functions as f64
    } else {
        0.0
    };

    // Create complexity distribution
    let mut complexity_distribution = HashMap::new();
    complexity_distribution.insert("low".to_string(), 0);
    complexity_distribution.insert("moderate".to_string(), 0);
    complexity_distribution.insert("high".to_string(), 0);
    complexity_distribution.insert("very_high".to_string(), 0);

    for (_, func) in &functions {
        let level = match func.complexity_level {
            ComplexityLevel::Low => "low",
            ComplexityLevel::Moderate => "moderate",
            ComplexityLevel::High => "high",
            ComplexityLevel::VeryHigh => "very_high",
        };
        *complexity_distribution.get_mut(level).unwrap() += 1;
    }

    let result = ComplexityAnalysisResult {
        functions,
        total_functions,
        average_complexity,
        max_complexity,
        max_complexity_function,
        complexity_distribution,
    };

    DynTrait::from_value(result)
}

#[sabi_extern_fn]
fn generate_report_pf(pf_results: PFDependencies, user_params: &UserParameters) -> BoxedPFResult<'static> {
    // Get complexity analysis from previous plugin function
    let analysis_key = QualPFID {
        plugin_id: PLUGIN_NAME.into(),
        pf_id: "analyze_complexity".into(),
    };
    
    let analysis = pf_results.get(&analysis_key)
        .expect("Missing complexity analysis result");
    
    let _analysis = unsafe { 
        analysis.unchecked_downcast_as::<ComplexityAnalysisResult>() 
    };

    // Get output path and format from user parameters
    let output_path = user_params.get("output_path")
        .map(|p| unsafe { p.unchecked_downcast_as::<RString>() }.as_str())
        .unwrap_or("./reports");

    let format = user_params.get("format")
        .map(|p| unsafe { p.unchecked_downcast_as::<RString>() }.as_str())
        .unwrap_or("json");

    // Generate report path based on format
    let report_path = match format {
        "html" => format!("{}/complexity_report.html", output_path),
        "csv" => format!("{}/complexity_report.csv", output_path),
        _ => format!("{}/complexity_report.json", output_path),
    };

    // In a real implementation, this would:
    // 1. Generate formatted report based on the analysis data
    // 2. Create visualizations for HTML format
    // 3. Write the report to the specified path
    // 4. Return the actual report path

    let result = ReportResult {
        report_path,
        format: format.to_string(),
    };

    DynTrait::from_value(result)
}

/// Analyze complexity for a single function using McCabe's formula and decision point counting
fn analyze_function_complexity(cfg: &ControlFlowGraph, include_cognitive: bool) -> FunctionComplexity {
    // Calculate cyclomatic complexity using McCabe's formula: CC = E - N + 2P
    // For a single function, P = 1 (one connected component)

    // Count nodes - exclude entry and exit blocks from the count
    let num_nodes = cfg.blocks.len();

    // Count edges
    let num_edges: usize = cfg.blocks.values().map(|block| block.succs.len()).sum();

    // Apply McCabe's formula: CC = E - N + 2P (where P = 1)
    let cyclomatic_complexity = if num_nodes > 0 {
        if num_nodes == 2 && num_edges == 1 {
            // Empty function with just entry -> exit
            1
        } else {
            // Use saturating_sub to avoid potential overflow
            num_edges.saturating_add(2).saturating_sub(num_nodes).max(1)
        }
    } else {
        1
    };

    // Collect decision points and calculate cognitive complexity
    let mut decision_points = Vec::new();
    let mut cognitive_complexity = 0;
    let mut max_nesting_depth = 0;
    let mut lines_of_code = 0;

    // Analyze each block in the CFG
    for (block_id, block) in &cfg.blocks {
        // Skip entry and exit blocks for statement analysis
        if *block_id == cfg.entry || block.stmts.is_empty() {
            continue;
        }

        // Check if this is an exit block (no successors)
        let is_exit = block.succs.is_empty();

        for stmt in &block.stmts {
            // Skip empty statements in exit blocks
            if is_exit {
                if let Stmt::Empty(e) = &stmt.stmt {
                    if e.implicit {
                        continue;
                    }
                }
            }

            lines_of_code += 1;

            // Process each statement for decision points (simplified version)
            if let Some(dp) = analyze_statement_for_decision_point(&stmt.stmt, 0) {
                decision_points.push(dp);
            }

            if include_cognitive {
                let (cognitive_score, nesting) = calculate_cognitive_complexity(&stmt.stmt, 0);
                cognitive_complexity += cognitive_score;
                max_nesting_depth = max_nesting_depth.max(nesting);
            }
        }
    }

    // Determine complexity level
    let complexity_level = match cyclomatic_complexity {
        1..=5 => ComplexityLevel::Low,
        6..=10 => ComplexityLevel::Moderate,
        11..=20 => ComplexityLevel::High,
        _ => ComplexityLevel::VeryHigh,
    };

    FunctionComplexity {
        cyclomatic_complexity,
        cognitive_complexity,
        lines_of_code,
        decision_points,
        complexity_level,
        nesting_depth_max: max_nesting_depth,
    }
}

/// Analyze a statement for decision points (simplified version without fset/objs)
fn analyze_statement_for_decision_point(stmt: &Stmt, nesting_level: usize) -> Option<DecisionPoint> {
    match stmt {
        Stmt::If(_) => Some(DecisionPoint {
            line: 0, // Would need fset to get real line numbers
            stmt_type: "if".to_string(),
            nesting_level,
        }),
        Stmt::For(_) => Some(DecisionPoint {
            line: 0,
            stmt_type: "for".to_string(),
            nesting_level,
        }),
        Stmt::Switch(_) => Some(DecisionPoint {
            line: 0,
            stmt_type: "switch".to_string(),
            nesting_level,
        }),
        Stmt::Range(_) => Some(DecisionPoint {
            line: 0,
            stmt_type: "range".to_string(),
            nesting_level,
        }),
        Stmt::TypeSwitch(_) => Some(DecisionPoint {
            line: 0,
            stmt_type: "typeswitch".to_string(),
            nesting_level,
        }),
        Stmt::Case(_) => Some(DecisionPoint {
            line: 0,
            stmt_type: "case".to_string(),
            nesting_level,
        }),
        Stmt::Select(_) => Some(DecisionPoint {
            line: 0,
            stmt_type: "select".to_string(),
            nesting_level,
        }),
        _ => None,
    }
}

/// Calculate cognitive complexity for a statement (simplified version)
fn calculate_cognitive_complexity(stmt: &Stmt, nesting_level: usize) -> (usize, usize) {
    match stmt {
        Stmt::If(_) => {
            let complexity = 1 + nesting_level; // Base complexity + nesting penalty
            let max_nesting = nesting_level + 1;
            (complexity, max_nesting)
        }
        Stmt::For(_) | Stmt::Range(_) => {
            let complexity = 1 + nesting_level;
            let max_nesting = nesting_level + 1;
            (complexity, max_nesting)
        }
        Stmt::Switch(_) => {
            let complexity = 1 + nesting_level;
            let max_nesting = nesting_level + 1;
            (complexity, max_nesting)
        }
        _ => (0, nesting_level),
    }
}

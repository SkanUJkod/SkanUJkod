use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use cfg::ast::parse_project;
use cfg::cfg::{build_cfgs_for_file, ControlFlowGraph};
use go_parser::ast::{Node, Stmt};

use crate::helpers::{go_utils, ComplexityLevel};

/// Structure representing complexity analysis for a single function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionComplexity {
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
    pub lines_of_code: usize,
    pub decision_points: Vec<DecisionPoint>,
    pub complexity_level: ComplexityLevel,
    pub nesting_depth_max: usize,
}

/// Structure representing a decision point in the code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPoint {
    pub line: usize,
    pub stmt_type: String,
    pub nesting_level: usize,
}

/// Structure representing complexity analysis for the entire project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectComplexity {
    pub functions: HashMap<String, FunctionComplexity>,
    pub total_functions: usize,
    pub average_complexity: f64,
    pub max_complexity: usize,
    pub max_complexity_function: String,
    pub complexity_distribution: HashMap<String, usize>,
    pub files_analyzed: usize,
}

/// Configuration options for complexity analysis
#[derive(Debug, Clone)]
pub struct ComplexityOptions {
    pub verbose: bool,
    pub include_cognitive: bool,
    pub max_allowed_complexity: usize,
    pub fail_on_high_complexity: bool,
}

impl Default for ComplexityOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            include_cognitive: true,
            max_allowed_complexity: 10,
            fail_on_high_complexity: false,
        }
    }
}

/// Main entry point for analyzing cyclomatic complexity
///
/// # Arguments
/// * `project_path` - Path to the Go project to analyze
///
/// # Returns
/// * `Result<ProjectComplexity>` - Analysis results or error
pub fn analyze_cyclomatic_complexity(project_path: &Path) -> Result<ProjectComplexity> {
    analyze_cyclomatic_complexity_with_options(project_path, &ComplexityOptions::default())
}

/// Analyze cyclomatic complexity with custom options
///
/// # Arguments
/// * `project_path` - Path to the Go project to analyze
/// * `options` - Custom analysis options
///
/// # Returns
/// * `Result<ProjectComplexity>` - Analysis results or error
pub fn analyze_cyclomatic_complexity_with_options(
    project_path: &Path,
    options: &ComplexityOptions,
) -> Result<ProjectComplexity> {
    if !project_path.exists() {
        anyhow::bail!("Project path does not exist: {}", project_path.display());
    }

    // Check if Go is installed
    go_utils::check_go_installation().context("Go installation check failed")?;

    // Parse the project and build CFGs
    let (fset, objs, files) = parse_project(project_path).context("Failed to parse Go project")?;

    if files.is_empty() {
        anyhow::bail!("No Go files found in the project");
    }

    let mut all_cfgs: HashMap<String, ControlFlowGraph> = HashMap::new();
    for pf in &files {
        let file_cfgs = build_cfgs_for_file(&fset, &objs, &pf.ast);
        all_cfgs.extend(file_cfgs);
    }

    if all_cfgs.is_empty() {
        anyhow::bail!("No functions found in the project");
    }

    // Analyze complexity for each function
    let mut functions = HashMap::new();
    let mut max_complexity = 0;
    let mut max_complexity_function = String::new();

    for (func_name, cfg) in &all_cfgs {
        let func_complexity = analyze_function_complexity(cfg, &objs, &fset, options)?;

        if func_complexity.cyclomatic_complexity > max_complexity {
            max_complexity = func_complexity.cyclomatic_complexity;
            max_complexity_function = func_name.clone();
        }

        functions.insert(func_name.clone(), func_complexity);
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

    let project_complexity = ProjectComplexity {
        functions,
        total_functions,
        average_complexity,
        max_complexity,
        max_complexity_function: max_complexity_function.clone(),
        complexity_distribution,
        files_analyzed: files.len(),
    };

    // Check if we should fail on high complexity
    if options.fail_on_high_complexity && max_complexity > options.max_allowed_complexity {
        anyhow::bail!(
            "Function '{}' has cyclomatic complexity {} which exceeds the maximum allowed complexity {}",
            max_complexity_function,
            max_complexity,
            options.max_allowed_complexity
        );
    }

    Ok(project_complexity)
}

/// Analyze complexity for a single function
fn analyze_function_complexity(
    cfg: &ControlFlowGraph,
    objs: &go_parser::AstObjects,
    fset: &go_parser::FileSet,
    options: &ComplexityOptions,
) -> Result<FunctionComplexity> {
    // Calculate cyclomatic complexity using McCabe's formula: CC = E - N + 2P
    // For a single function, P = 1 (one connected component)

    // Count nodes - exclude entry and exit blocks from the count
    let num_nodes = cfg.blocks.len();

    // Count edges
    let num_edges: usize = cfg.blocks.values().map(|block| block.succs.len()).sum();

    // Apply McCabe's formula: CC = E - N + 2P (where P = 1)
    // But be careful to avoid overflow: use max(E + 2 - N, 1) to ensure CC is at least 1
    let cyclomatic_complexity = if num_nodes > 0 {
        // For proper calculation, we need to handle empty functions
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

            // Process each statement for decision points
            process_statement_for_decision_points(&stmt.stmt, objs, fset, 0, &mut decision_points);

            if options.include_cognitive {
                let (cognitive_score, nesting) =
                    calculate_cognitive_complexity(&stmt.stmt, objs, 0);
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

    Ok(FunctionComplexity {
        cyclomatic_complexity,
        cognitive_complexity,
        lines_of_code,
        decision_points,
        complexity_level,
        nesting_depth_max: max_nesting_depth,
    })
}

/// Process a statement to collect all decision points (including nested ones)
fn process_statement_for_decision_points(
    stmt: &Stmt,
    objs: &go_parser::AstObjects,
    fset: &go_parser::FileSet,
    nesting_level: usize,
    decision_points: &mut Vec<DecisionPoint>,
) {
    // First, check if this statement itself is a decision point
    match stmt {
        Stmt::For(_) => {
            let pos = stmt.pos(objs);
            if let Some(position) = fset.position(pos) {
                decision_points.push(DecisionPoint {
                    line: position.line,
                    stmt_type: "for".to_string(),
                    nesting_level,
                });
            }
        }
        Stmt::Switch(_) => {
            let pos = stmt.pos(objs);
            if let Some(position) = fset.position(pos) {
                decision_points.push(DecisionPoint {
                    line: position.line,
                    stmt_type: "switch".to_string(),
                    nesting_level,
                });
            }
        }
        _ => {
            if let Some(dp) = analyze_statement(stmt, objs, fset, nesting_level) {
                decision_points.push(dp);
            }
        }
    }

    // Then recursively process any child statements
    match stmt {
        Stmt::If(if_stmt) => {
            // Process the 'then' block
            for s in &if_stmt.body.list {
                process_statement_for_decision_points(
                    s,
                    objs,
                    fset,
                    nesting_level + 1,
                    decision_points,
                );
            }

            // Process the 'else' block if it exists
            if let Some(els) = &if_stmt.els {
                process_statement_for_decision_points(
                    els,
                    objs,
                    fset,
                    nesting_level + 1,
                    decision_points,
                );
            }
        }
        Stmt::For(for_stmt) => {
            // Process the loop body
            for s in &for_stmt.body.list {
                process_statement_for_decision_points(
                    s,
                    objs,
                    fset,
                    nesting_level + 1,
                    decision_points,
                );
            }
        }
        Stmt::Range(range_stmt) => {
            // Process the range loop body
            for s in &range_stmt.body.list {
                process_statement_for_decision_points(
                    s,
                    objs,
                    fset,
                    nesting_level + 1,
                    decision_points,
                );
            }
        }
        Stmt::Switch(switch_stmt) => {
            // Process the switch body, which contains case clauses
            for s in &switch_stmt.body.list {
                process_statement_for_decision_points(
                    s,
                    objs,
                    fset,
                    nesting_level + 1,
                    decision_points,
                );
            }
        }
        Stmt::TypeSwitch(type_switch) => {
            // Process the type switch body
            for s in &type_switch.body.list {
                process_statement_for_decision_points(
                    s,
                    objs,
                    fset,
                    nesting_level + 1,
                    decision_points,
                );
            }
        }
        Stmt::Case(case_stmt) => {
            // Process the case body
            for s in &case_stmt.body {
                process_statement_for_decision_points(
                    s,
                    objs,
                    fset,
                    nesting_level + 1,
                    decision_points,
                );
            }
        }
        Stmt::Block(block) => {
            // Process all statements in the block
            for s in &block.list {
                process_statement_for_decision_points(
                    s,
                    objs,
                    fset,
                    nesting_level,
                    decision_points,
                );
            }
        }
        _ => {} // Other statement types don't have nested statements
    }
}

/// Analyze a statement for decision points
fn analyze_statement(
    stmt: &Stmt,
    objs: &go_parser::AstObjects,
    fset: &go_parser::FileSet,
    nesting_level: usize,
) -> Option<DecisionPoint> {
    match stmt {
        Stmt::If(_) => {
            let pos = stmt.pos(objs);
            if let Some(position) = fset.position(pos) {
                Some(DecisionPoint {
                    line: position.line,
                    stmt_type: "if".to_string(),
                    nesting_level,
                })
            } else {
                None
            }
        }
        Stmt::For(_) => {
            let pos = stmt.pos(objs);
            if let Some(position) = fset.position(pos) {
                Some(DecisionPoint {
                    line: position.line,
                    stmt_type: "for".to_string(),
                    nesting_level,
                })
            } else {
                None
            }
        }
        Stmt::Switch(_) => {
            let pos = stmt.pos(objs);
            if let Some(position) = fset.position(pos) {
                Some(DecisionPoint {
                    line: position.line,
                    stmt_type: "switch".to_string(),
                    nesting_level,
                })
            } else {
                None
            }
        }
        Stmt::Range(_) => {
            let pos = stmt.pos(objs);
            if let Some(position) = fset.position(pos) {
                Some(DecisionPoint {
                    line: position.line,
                    stmt_type: "range".to_string(),
                    nesting_level,
                })
            } else {
                None
            }
        }
        Stmt::TypeSwitch(_) => {
            let pos = stmt.pos(objs);
            if let Some(position) = fset.position(pos) {
                Some(DecisionPoint {
                    line: position.line,
                    stmt_type: "typeswitch".to_string(),
                    nesting_level,
                })
            } else {
                None
            }
        }
        Stmt::Case(_) => {
            let pos = stmt.pos(objs);
            if let Some(position) = fset.position(pos) {
                Some(DecisionPoint {
                    line: position.line,
                    stmt_type: "case".to_string(),
                    nesting_level,
                })
            } else {
                None
            }
        }
        Stmt::Select(_) => {
            let pos = stmt.pos(objs);
            if let Some(position) = fset.position(pos) {
                Some(DecisionPoint {
                    line: position.line,
                    stmt_type: "select".to_string(),
                    nesting_level,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Calculate cognitive complexity for a statement
fn calculate_cognitive_complexity(
    stmt: &Stmt,
    objs: &go_parser::AstObjects,
    nesting_level: usize,
) -> (usize, usize) {
    match stmt {
        Stmt::If(if_stmt) => {
            let mut complexity = 1 + nesting_level; // Base complexity + nesting penalty
            let mut max_nesting = nesting_level + 1;

            // Analyze then branch
            for s in &if_stmt.body.list {
                let (c, n) = calculate_cognitive_complexity(s, objs, nesting_level + 1);
                complexity += c;
                max_nesting = max_nesting.max(n);
            }

            // Analyze else branch if exists
            if let Some(else_stmt) = &if_stmt.els {
                let (c, n) = calculate_cognitive_complexity(else_stmt, objs, nesting_level + 1);
                complexity += c;
                max_nesting = max_nesting.max(n);
            }

            (complexity, max_nesting)
        }
        Stmt::For(for_stmt) => {
            let mut complexity = 1 + nesting_level;
            let mut max_nesting = nesting_level + 1;

            for s in &for_stmt.body.list {
                let (c, n) = calculate_cognitive_complexity(s, objs, nesting_level + 1);
                complexity += c;
                max_nesting = max_nesting.max(n);
            }

            (complexity, max_nesting)
        }
        Stmt::Range(range_stmt) => {
            let mut complexity = 1 + nesting_level;
            let mut max_nesting = nesting_level + 1;

            for s in &range_stmt.body.list {
                let (c, n) = calculate_cognitive_complexity(s, objs, nesting_level + 1);
                complexity += c;
                max_nesting = max_nesting.max(n);
            }

            (complexity, max_nesting)
        }
        Stmt::Switch(switch_stmt) => {
            let mut complexity = 1 + nesting_level;
            let mut max_nesting = nesting_level + 1;

            for s in &switch_stmt.body.list {
                let (c, n) = calculate_cognitive_complexity(s, objs, nesting_level + 1);
                complexity += c;
                max_nesting = max_nesting.max(n);
            }

            (complexity, max_nesting)
        }
        Stmt::Block(block_stmt) => {
            let mut complexity = 0;
            let mut max_nesting = nesting_level;

            for s in &block_stmt.list {
                let (c, n) = calculate_cognitive_complexity(s, objs, nesting_level);
                complexity += c;
                max_nesting = max_nesting.max(n);
            }

            (complexity, max_nesting)
        }
        _ => (0, nesting_level),
    }
}

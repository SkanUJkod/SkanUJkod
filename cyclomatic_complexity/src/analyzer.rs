use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use cfg::ast::parse_project;
use cfg::cfg::{build_cfgs_for_file, ControlFlowGraph};
use go_parser::ast::{Stmt, Node};
use go_parser::Token;

use crate::helpers::{ComplexityLevel, go_utils};

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
pub fn analyze_cyclomatic_complexity(project_path: &Path) -> Result<ProjectComplexity> {
    analyze_cyclomatic_complexity_with_options(project_path, &ComplexityOptions::default())
}

/// Analyze cyclomatic complexity with custom options
pub fn analyze_cyclomatic_complexity_with_options(
    project_path: &Path,
    options: &ComplexityOptions,
) -> Result<ProjectComplexity> {
    if !project_path.exists() {
        anyhow::bail!("Project path does not exist: {}", project_path.display());
    }

    // Check if Go is installed
    go_utils::check_go_installation().context("Go installation check failed")?;

    if options.verbose {
        println!("🔍 Analyzing cyclomatic complexity for: {}", project_path.display());
    }

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

    if options.verbose {
        println!("📊 Found {} functions to analyze", all_cfgs.len());
        println!("📄 Found {} Go files", files.len());
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
    let total_complexity: usize = functions.values()
        .map(|f| f.cyclomatic_complexity)
        .sum();
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

    for func in functions.values() {
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

    if options.verbose {
        print_complexity_report(&project_complexity);
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
    let num_edges: usize = cfg.blocks.values()
        .map(|block| block.succs.len())
        .sum();
    
    // Apply McCabe's formula: CC = E - N + 2P (where P = 1)
    let cyclomatic_complexity = if num_nodes > 0 {
        // For proper calculation, we need to handle empty functions
        if num_nodes == 2 && num_edges == 1 {
            // Empty function with just entry -> exit
            1
        } else {
            num_edges - num_nodes + 2
        }
    } else {
        1
    };

    // Alternative calculation based on decision points (for validation)
    let mut decision_count = 0;
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
            
            // Count decision points for alternative CC calculation
            if is_decision_statement(&stmt.stmt) {
                decision_count += 1;
            }
            
            if let Some(decision_point) = analyze_statement(&stmt.stmt, objs, fset, 0) {
                decision_points.push(decision_point);
            }

            if options.include_cognitive {
                let (cognitive_score, nesting) = calculate_cognitive_complexity(&stmt.stmt, objs, 0);
                cognitive_complexity += cognitive_score;
                max_nesting_depth = max_nesting_depth.max(nesting);
            }
        }
        
        // Also count decision based on branching in CFG
        if block.succs.len() > 1 {
            // This block has multiple successors, indicating a decision point
            // This helps catch implicit decisions not represented in statements
        }
    }

    // Validate cyclomatic complexity calculation
    // Alternative formula: CC = 1 + number of decision points
    let cc_from_decisions = 1 + decision_count;
    
    // Use the graph-based calculation as primary, but log if there's a significant difference
    if options.verbose && (cyclomatic_complexity as i32 - cc_from_decisions as i32).abs() > 1 {
        eprintln!("Note: Graph-based CC ({}) differs from decision-based CC ({})", 
                  cyclomatic_complexity, cc_from_decisions);
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

/// Check if a statement is a decision point
fn is_decision_statement(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::If(_) => true,
        Stmt::For(f) => f.cond.is_some(), // Only count loops with conditions
        Stmt::Range(_) => true,
        Stmt::Switch(_) => true,
        Stmt::TypeSwitch(_) => true,
        Stmt::Case(_) => true, // Each case is a decision point
        Stmt::Select(_) => true,
        _ => false,
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

/// Print complexity report
pub fn print_complexity_report(complexity: &ProjectComplexity) {
    println!("\n╔══════════════════════════════════════════════╗");
    println!("║      Cyclomatic Complexity Report            ║");
    println!("╚══════════════════════════════════════════════╝\n");

    println!("📊 Summary:");
    println!("  Files analyzed: {}", complexity.files_analyzed);
    println!("  Functions found: {}", complexity.total_functions);
    println!("  Average complexity: {:.2}", complexity.average_complexity);
    println!("  Maximum complexity: {} ({})", complexity.max_complexity, complexity.max_complexity_function);
    println!();

    println!("📈 Complexity Distribution:");
    println!("  Low (1-5): {} functions", complexity.complexity_distribution.get("low").unwrap_or(&0));
    println!("  Moderate (6-10): {} functions", complexity.complexity_distribution.get("moderate").unwrap_or(&0));
    println!("  High (11-20): {} functions", complexity.complexity_distribution.get("high").unwrap_or(&0));
    println!("  Very High (>20): {} functions", complexity.complexity_distribution.get("very_high").unwrap_or(&0));
    println!();

    // Sort functions by complexity (highest first)
    let mut sorted_functions: Vec<_> = complexity.functions.iter().collect();
    sorted_functions.sort_by(|a, b| {
        b.1.cyclomatic_complexity.cmp(&a.1.cyclomatic_complexity)
    });

    println!("🔍 Top 10 Most Complex Functions:");
    println!("─────────────────────────────────────────────────");
    
    for (func_name, func_complexity) in sorted_functions.iter().take(10) {
        let symbol = match func_complexity.complexity_level {
            ComplexityLevel::Low => "✅",
            ComplexityLevel::Moderate => "⚠️ ",
            ComplexityLevel::High => "🔶",
            ComplexityLevel::VeryHigh => "🔴",
        };
        
        println!("{} {} - CC: {}, Cognitive: {}, LOC: {}",
            symbol,
            func_name,
            func_complexity.cyclomatic_complexity,
            func_complexity.cognitive_complexity,
            func_complexity.lines_of_code
        );
        
        // Show decision points for highly complex functions
        if func_complexity.cyclomatic_complexity > 10 && !func_complexity.decision_points.is_empty() {
            println!("   Decision points:");
            for (i, dp) in func_complexity.decision_points.iter().enumerate() {
                if i >= 3 {
                    println!("   ... and {} more", func_complexity.decision_points.len() - 3);
                    break;
                }
                println!("   - Line {}: {} (nesting: {})", dp.line, dp.stmt_type, dp.nesting_level);
            }
        }
        println!();
    }

    println!("─────────────────────────────────────────────────");
    
    // Provide recommendations
    if complexity.max_complexity > 20 {
        println!("⚠️  WARNING: Some functions have very high complexity!");
        println!("   Consider refactoring functions with complexity > 10");
    } else if complexity.average_complexity > 10.0 {
        println!("⚠️  WARNING: Average complexity is high!");
        println!("   Consider breaking down complex functions");
    } else {
        println!("✅ Overall complexity is within acceptable limits");
    }
}

/// Export complexity report to file
pub fn export_complexity_report(
    complexity: &ProjectComplexity,
    output_path: &Path,
    format: ExportFormat,
) -> Result<()> {
    match format {
        ExportFormat::Json => {
            let report = serde_json::to_string_pretty(complexity)?;
            fs::write(output_path, report)?;
        }
        ExportFormat::Html => {
            let html = generate_html_report(complexity)?;
            fs::write(output_path, html)?;
        }
        ExportFormat::Csv => {
            let csv = generate_csv_report(complexity);
            fs::write(output_path, csv)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Html,
    Csv,
}

fn generate_html_report(complexity: &ProjectComplexity) -> Result<String> {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<title>Cyclomatic Complexity Report</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
    html.push_str("table { border-collapse: collapse; width: 100%; }\n");
    html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
    html.push_str("th { background-color: #4CAF50; color: white; }\n");
    html.push_str(".low { background-color: #4CAF50; color: white; }\n");
    html.push_str(".moderate { background-color: #ff9800; color: white; }\n");
    html.push_str(".high { background-color: #f44336; color: white; }\n");
    html.push_str(".very-high { background-color: #9c27b0; color: white; }\n");
    html.push_str("</style>\n</head>\n<body>\n");
    
    html.push_str("<h1>Cyclomatic Complexity Report</h1>\n");
    html.push_str(&format!("<p>Total Functions: {}</p>\n", complexity.total_functions));
    html.push_str(&format!("<p>Average Complexity: {:.2}</p>\n", complexity.average_complexity));
    html.push_str(&format!("<p>Maximum Complexity: {} ({})</p>\n", 
        complexity.max_complexity, complexity.max_complexity_function));
    
    html.push_str("<h2>Function Details</h2>\n");
    html.push_str("<table>\n<tr>\n");
    html.push_str("<th>Function</th>\n");
    html.push_str("<th>Cyclomatic</th>\n");
    html.push_str("<th>Cognitive</th>\n");
    html.push_str("<th>LOC</th>\n");
    html.push_str("<th>Level</th>\n");
    html.push_str("</tr>\n");
    
    let mut sorted_functions: Vec<_> = complexity.functions.iter().collect();
    sorted_functions.sort_by(|a, b| {
        b.1.cyclomatic_complexity.cmp(&a.1.cyclomatic_complexity)
    });
    
    for (name, func) in sorted_functions {
        let level_class = match func.complexity_level {
            ComplexityLevel::Low => "low",
            ComplexityLevel::Moderate => "moderate",
            ComplexityLevel::High => "high",
            ComplexityLevel::VeryHigh => "very-high",
        };
        
        html.push_str(&format!(
            "<tr>\n<td>{}</td>\n<td>{}</td>\n<td>{}</td>\n<td>{}</td>\n<td class=\"{}\">{:?}</td>\n</tr>\n",
            name, func.cyclomatic_complexity, func.cognitive_complexity, 
            func.lines_of_code, level_class, func.complexity_level
        ));
    }
    
    html.push_str("</table>\n</body>\n</html>");
    Ok(html)
}

fn generate_csv_report(complexity: &ProjectComplexity) -> String {
    let mut csv = String::from("Function,Cyclomatic Complexity,Cognitive Complexity,Lines of Code,Level\n");
    
    let mut sorted_functions: Vec<_> = complexity.functions.iter().collect();
    sorted_functions.sort_by_key(|(name, _)| name.as_str());
    
    for (name, func) in sorted_functions {
        csv.push_str(&format!(
            "{},{},{},{},{:?}\n",
            name, func.cyclomatic_complexity, func.cognitive_complexity,
            func.lines_of_code, func.complexity_level
        ));
    }
    
    csv
}
use crate::core::BranchCoverageOptions;
use crate::helpers::SourceLocation;
use anyhow::{Context, Result};
use cfg::ControlFlowGraph;
use go_parser::ast::Node;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Add missing type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BranchType {
    If,
    Loop,
    Switch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchPoint {
    pub id: u32,
    pub location: SourceLocation,
    pub branch_type: BranchType,
    pub true_taken: bool,
    pub false_taken: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub file: String,
    pub branches: Vec<BranchPoint>,
    pub total_branches: usize,
    pub covered_branches: usize,
}

/// Data about instrumentation added to the code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentationData {
    pub instrumented_functions: HashMap<String, FunctionInstrumentation>,
    pub branch_mapping: HashMap<String, BranchMapping>,
    pub total_branches: usize,
}

/// Information about instrumentation added to a single function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInstrumentation {
    pub function_name: String,
    pub file_path: String,
    pub branches: Vec<InstrumentedBranch>,
    pub instrumentation_points: Vec<InstrumentationPoint>,
}

/// Information about an instrumented branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentedBranch {
    pub branch_id: String,
    pub line: usize,
    pub branch_type: String,
    pub condition: String,
    pub instrumentation_code: String,
}

/// A point where instrumentation code was inserted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentationPoint {
    pub line: usize,
    pub column: usize,
    pub code: String,
    pub branch_id: String,
}

/// Maps branch IDs to their locations and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchMapping {
    pub branch_id: String,
    pub function_name: String,
    pub file_path: String,
    pub line: usize,
    pub branch_type: String,
}

#[derive(Debug, Clone)]
pub struct BranchInstrumentation {
    pub options: BranchCoverageOptions,
    pub branch_data: HashMap<u32, BranchPoint>,
    pub current_file: String,
}

impl BranchInstrumentation {
    pub fn new(options: BranchCoverageOptions) -> Self {
        Self {
            options,
            branch_data: HashMap::new(),
            current_file: String::new(),
        }
    }

    pub fn instrument_file(&mut self, file_path: &Path) -> Result<String> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        self.current_file = file_path.to_string_lossy().to_string();

        // For now, return a simple instrumented version
        // In a complete implementation, you would parse and instrument the Go code
        let instrumented = format!(
            "// Instrumented version of {}\n{}",
            file_path.display(),
            content
        );

        Ok(instrumented)
    }

    pub fn get_coverage_data(&self) -> &HashMap<u32, BranchPoint> {
        &self.branch_data
    }

    pub fn export_coverage_report(&self, output_path: &Path) -> Result<()> {
        let report = CoverageReport {
            file: self.current_file.clone(),
            branches: self.branch_data.values().cloned().collect(),
            total_branches: self.branch_data.len(),
            covered_branches: self
                .branch_data
                .values()
                .filter(|bp| bp.true_taken || bp.false_taken)
                .count(),
        };

        let json = serde_json::to_string_pretty(&report)
            .with_context(|| "Failed to serialize coverage report")?;

        fs::write(output_path, json).with_context(|| {
            format!(
                "Failed to write coverage report to {}",
                output_path.display()
            )
        })?;

        Ok(())
    }
}

/// Instrument Go code for branch coverage analysis
pub fn instrument_for_branch_coverage(
    project_path: &Path,
    output_path: &Path,
    cfgs: &HashMap<String, ControlFlowGraph>,
    fset: &go_parser::FileSet,
    objs: &go_parser::AstObjects,
    options: &BranchCoverageOptions,
) -> Result<InstrumentationData> {
    if options.verbose {
        println!("Instrumenting code for branch coverage...");
    }

    // Copy the original project to the output directory
    copy_project(project_path, output_path)?;

    // Create instrumentation data structure
    let mut instrumentation_data = InstrumentationData {
        instrumented_functions: HashMap::new(),
        branch_mapping: HashMap::new(),
        total_branches: 0,
    };

    // Process each function's CFG to add instrumentation
    for (func_name, cfg) in cfgs {
        let function_instrumentation =
            instrument_function(func_name, cfg, fset, objs, output_path, options)?;

        // Update the total branch count
        instrumentation_data.total_branches += function_instrumentation.branches.len();

        // Add branch mappings
        for branch in &function_instrumentation.branches {
            instrumentation_data.branch_mapping.insert(
                branch.branch_id.clone(),
                BranchMapping {
                    branch_id: branch.branch_id.clone(),
                    function_name: func_name.clone(),
                    file_path: function_instrumentation.file_path.clone(),
                    line: branch.line,
                    branch_type: branch.branch_type.clone(),
                },
            );
        }

        instrumentation_data
            .instrumented_functions
            .insert(func_name.clone(), function_instrumentation);
    }

    if options.verbose {
        println!(
            "Instrumented {} functions with {} total branches",
            instrumentation_data.instrumented_functions.len(),
            instrumentation_data.total_branches
        );
    }

    Ok(instrumentation_data)
}

/// Instrument a single function for branch coverage
fn instrument_function(
    func_name: &str,
    cfg: &ControlFlowGraph,
    fset: &go_parser::FileSet,
    objs: &go_parser::AstObjects,
    _output_path: &Path,
    options: &BranchCoverageOptions,
) -> Result<FunctionInstrumentation> {
    let mut branches = Vec::new();
    let mut instrumentation_points = Vec::new();
    let mut seen_branches = std::collections::HashSet::new();

    if options.verbose {
        println!("Instrumenting function: {}", func_name);
    }

    // Find all branch points in the CFG
    for (block_id, block) in &cfg.blocks {
        // Skip blocks that don't represent real branch points
        if block.succs.len() <= 1 {
            continue;
        }

        // Get precise line number from the branching statement
        let line = get_precise_branch_line(block, fset, objs);

        if options.verbose && line == 0 {
            println!(
                "Warning: Block {} has no precise line information",
                block_id
            );
        }

        // Determine branch type and condition from the branching statement
        let (branch_type, condition) = analyze_block_branch_info(block, objs);

        // Create branches for each successor
        for (succ_idx, _succ_id) in block.succs.iter().enumerate() {
            let branch_id = format!("{}:{}:{}", func_name, block_id, succ_idx);

            // Avoid duplicates
            if seen_branches.insert(branch_id.clone()) {
                let instrumentation_code = generate_branch_instrumentation(&branch_id);

                // For switch statements, create more specific conditions
                let specific_condition =
                    create_specific_condition(&branch_type, &condition, succ_idx);

                branches.push(InstrumentedBranch {
                    branch_id: branch_id.clone(),
                    line,
                    branch_type: branch_type.clone(),
                    condition: specific_condition,
                    instrumentation_code: instrumentation_code.clone(),
                });

                // Add instrumentation point with precise positioning
                instrumentation_points.push(InstrumentationPoint {
                    line,
                    column: get_precise_column(block, fset, objs),
                    code: instrumentation_code,
                    branch_id,
                });
            }
        }
    }

    if options.verbose {
        println!(
            "Found {} branches in function {}",
            branches.len(),
            func_name
        );
    }

    // Use main.go as default file path since we're working with example-go
    let file_path = "main.go".to_string();

    Ok(FunctionInstrumentation {
        function_name: func_name.to_string(),
        file_path,
        branches,
        instrumentation_points,
    })
}

/// Get precise line number for a branch block
fn get_precise_branch_line(
    block: &cfg::cfg::BasicBlock,
    fset: &go_parser::FileSet,
    objs: &go_parser::AstObjects,
) -> usize {
    // Look for the first statement that creates a branch
    for stmt in &block.stmts {
        if is_branching_statement(&stmt.stmt) {
            let pos = stmt.stmt.pos(objs);
            if let Some(position) = fset.position(pos) {
                return position.line;
            }
        }
    }

    // Fallback: get line from first statement with valid position
    for stmt in &block.stmts {
        let pos = stmt.stmt.pos(objs);
        if let Some(position) = fset.position(pos) {
            if position.line > 0 {
                return position.line;
            }
        }
    }

    // Last resort: use block's stored line number
    if block.start_line > 0 {
        block.start_line
    } else {
        block.end_line
    }
}

/// Get precise column number for instrumentation placement
fn get_precise_column(
    block: &cfg::cfg::BasicBlock,
    fset: &go_parser::FileSet,
    objs: &go_parser::AstObjects,
) -> usize {
    // Look for the first branching statement to get its column
    for stmt in &block.stmts {
        if is_branching_statement(&stmt.stmt) {
            let pos = stmt.stmt.pos(objs);
            if let Some(position) = fset.position(pos) {
                return position.column;
            }
        }
    }

    // Fallback to first statement's column
    if let Some(stmt) = block.stmts.first() {
        let pos = stmt.stmt.pos(objs);
        if let Some(position) = fset.position(pos) {
            return position.column;
        }
    }

    0
}

/// Check if a statement creates a branch
fn is_branching_statement(stmt: &go_parser::ast::Stmt) -> bool {
    use go_parser::ast::Stmt;
    matches!(
        stmt,
        Stmt::If(_)
            | Stmt::Switch(_)
            | Stmt::For(_)
            | Stmt::Range(_)
            | Stmt::TypeSwitch(_)
            | Stmt::Select(_)
    )
}

/// Analyze a block to determine its branch type and condition
fn analyze_block_branch_info(
    block: &cfg::cfg::BasicBlock,
    _objs: &go_parser::AstObjects,
) -> (String, String) {
    // Look for the branching statement that creates multiple successors
    for stmt in &block.stmts {
        if let Some(branch_info) = analyze_branch_statement_detailed(&stmt.stmt) {
            return branch_info;
        }
    }

    // Fallback for blocks without explicit branching statements
    match block.succs.len() {
        0 => ("terminal".to_string(), "end_of_function".to_string()),
        1 => ("sequential".to_string(), "fall_through".to_string()),
        2 => ("conditional".to_string(), "binary_branch".to_string()),
        _ => ("multi_way".to_string(), "switch_or_select".to_string()),
    }
}

/// Create specific condition text based on branch type and successor index
fn create_specific_condition(branch_type: &str, condition: &str, succ_idx: usize) -> String {
    match branch_type {
        "switch" | "type_switch" => {
            format!("{} (case {})", condition, succ_idx)
        }
        "select" => {
            format!("{} (channel {})", condition, succ_idx)
        }
        "if" => {
            if succ_idx == 0 {
                format!("{} (true)", condition)
            } else {
                format!("{} (false)", condition)
            }
        }
        _ => {
            if succ_idx == 0 {
                format!("{} (path {})", condition, succ_idx)
            } else {
                format!("{} (alternative {})", condition, succ_idx)
            }
        }
    }
}

/// Analyze a statement to determine its branch type and condition
fn analyze_branch_statement_detailed(stmt: &go_parser::ast::Stmt) -> Option<(String, String)> {
    use go_parser::ast::Stmt;

    match stmt {
        Stmt::If(_) => Some(("if".to_string(), "if_condition".to_string())),
        Stmt::Switch(_) => Some(("switch".to_string(), "switch_expression".to_string())),
        Stmt::TypeSwitch(_) => Some(("type_switch".to_string(), "type_assertion".to_string())),
        Stmt::For(_) => Some(("for".to_string(), "for_condition".to_string())),
        Stmt::Range(_) => Some(("range".to_string(), "range_expression".to_string())),
        Stmt::Select(_) => Some(("select".to_string(), "channel_operations".to_string())),
        _ => None,
    }
}

/// Copy the original project to the output directory
fn copy_project(source: &Path, destination: &Path) -> Result<()> {
    if destination.exists() {
        fs::remove_dir_all(destination)?;
    }
    fs::create_dir_all(destination)?;

    copy_directory_contents(source, destination)?;
    Ok(())
}

/// Recursively copy directory contents
fn copy_directory_contents(source: &Path, destination: &Path) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = destination.join(&file_name);

        if source_path.is_dir() {
            // Skip certain directories
            if let Some(dir_name) = file_name.to_str() {
                if dir_name == "target" || dir_name == ".git" || dir_name == "vendor" {
                    continue;
                }
            }
            fs::create_dir_all(&dest_path)?;
            copy_directory_contents(&source_path, &dest_path)?;
        } else {
            fs::copy(&source_path, &dest_path)?;
        }
    }
    Ok(())
}

/// Generate branch tracking code for a specific branch
fn generate_branch_instrumentation(branch_id: &str) -> String {
    format!("fmt.Printf(\"BRANCH_COV:{}\\n\")", branch_id)
}

/// Modify Go source files to add instrumentation
pub fn modify_source_files(
    output_path: &Path,
    instrumentation_data: &InstrumentationData,
) -> Result<()> {
    // Group instrumentation by file
    let mut files_to_instrument = HashMap::new();

    for (_func_name, func_instrumentation) in &instrumentation_data.instrumented_functions {
        let file_path = &func_instrumentation.file_path;

        files_to_instrument
            .entry(file_path.clone())
            .or_insert_with(Vec::new)
            .push(func_instrumentation);
    }

    // Process each file that needs instrumentation
    for (file_path, instrumentations) in files_to_instrument {
        let source_file = output_path.join(&file_path);

        if source_file.exists() {
            let content = fs::read_to_string(&source_file)?;
            let mut modified_content = content;

            // Combine all instrumentations for this file
            let mut all_branches = Vec::new();
            for instrumentation in &instrumentations {
                all_branches.extend(instrumentation.branches.clone());
            }

            // Create a combined instrumentation object for this file
            let combined_instrumentation = FunctionInstrumentation {
                function_name: "combined".to_string(),
                file_path: file_path.clone(),
                branches: all_branches,
                instrumentation_points: Vec::new(),
            };

            // Apply instrumentation
            modified_content =
                add_instrumentation_to_file(&modified_content, &combined_instrumentation)?;

            // Write the modified content back to the file
            fs::write(&source_file, modified_content).with_context(|| {
                format!(
                    "Failed to write instrumented file: {}",
                    source_file.display()
                )
            })?;
        }
    }

    Ok(())
}

/// Add instrumentation code to a Go source file
fn add_instrumentation_to_file(
    content: &str,
    instrumentation: &FunctionInstrumentation,
) -> Result<String> {
    let mut modified_content = content.to_string();

    // Skip if already instrumented to prevent duplicates
    if modified_content.contains("BRANCH_COV:") {
        return Ok(modified_content);
    }

    // Ensure we have fmt import
    if !modified_content.contains("\"fmt\"") {
        if modified_content.contains("import (") {
            modified_content = modified_content.replace("import (", "import (\n\t\"fmt\"");
        } else if modified_content.contains("import \"fmt\"") {
            // Already has fmt import, do nothing
        } else {
            modified_content =
                modified_content.replace("package main", "package main\n\nimport \"fmt\"");
        }
    }

    // Add branch tracking calls strategically
    modified_content = add_simplified_branch_tracking(&modified_content, instrumentation);

    Ok(modified_content)
}

/// Add simplified branch tracking
fn add_simplified_branch_tracking(
    content: &str,
    instrumentation: &FunctionInstrumentation,
) -> String {
    let mut modified = content.to_string();

    // Add instrumentation for each specific branch point
    for branch in &instrumentation.branches {
        // Extract function name and block info from branch_id (format: func:block:succ)
        let parts: Vec<&str> = branch.branch_id.split(':').collect();
        if parts.len() >= 3 {
            let func_name = parts[0];
            let _block_id = parts[1];
            let succ_idx: usize = parts[2].parse().unwrap_or(0);

            // Add specific instrumentation based on branch type and successor index
            modified = add_branch_specific_tracking(
                &modified,
                func_name,
                &branch.branch_type,
                branch.line,
                succ_idx,
                &branch.branch_id,
            );
        }
    }

    modified
}

/// Add tracking specific to branch type and location
fn add_branch_specific_tracking(
    content: &str,
    func_name: &str,
    branch_type: &str,
    target_line: usize,
    succ_idx: usize,
    branch_id: &str,
) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut in_target_function = false;
    let mut brace_count = 0;
    let mut line_count = 1;

    for line_content in lines {
        let should_add_tracking = if in_target_function {
            match branch_type {
                "if" => {
                    // For if statements, add tracking when we find the condition near target line
                    if line_content.trim().starts_with("if ")
                        && (line_count == target_line
                            || (line_count >= target_line - 2 && line_count <= target_line + 2))
                    {
                        if succ_idx == 0 && line_content.contains("{") {
                            true // True branch
                        } else {
                            false
                        }
                    } else if line_content.trim().starts_with("} else") && succ_idx == 1 {
                        true // False branch
                    } else {
                        false
                    }
                }
                "switch" => {
                    // For switch statements, add tracking in each case
                    if (line_content.contains("case ") || line_content.contains("default:"))
                        && (line_count >= target_line - 5 && line_count <= target_line + 10)
                    {
                        true
                    } else {
                        false
                    }
                }
                "range" | "for" => {
                    // For loops, add tracking at loop entry and in body
                    if (line_content.contains("for ") || line_content.contains("range "))
                        && (line_count == target_line
                            || (line_count >= target_line - 1 && line_count <= target_line + 1))
                    {
                        if succ_idx == 0 {
                            // Loop body
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                _ => {
                    // Generic: add tracking when we find any control structure near target line
                    if (line_content.contains("if ")
                        || line_content.contains("for ")
                        || line_content.contains("switch ")
                        || line_content.contains("case "))
                        && (line_count >= target_line - 2 && line_count <= target_line + 2)
                    {
                        true
                    } else {
                        false
                    }
                }
            }
        } else {
            false
        };

        result.push(line_content.to_string());

        // Check if we're entering the target function
        if line_content.contains(&format!("func {}", func_name))
            || line_content.contains(&format!("func {}(", func_name))
        {
            in_target_function = true;
            brace_count = 0;
        }

        // Track braces to know when we exit the function
        if in_target_function {
            brace_count += line_content.matches('{').count() as i32;
            brace_count -= line_content.matches('}').count() as i32;

            if brace_count <= 0 && line_content.contains('}') && !line_content.contains('{') {
                in_target_function = false;
            }
        }

        // Add tracking code if needed
        if should_add_tracking {
            let tracking_code = format!("    fmt.Printf(\"BRANCH_COV:{}\\n\")", branch_id);
            result.push(tracking_code);
        }

        line_count += 1;
    }

    result.join("\n")
}

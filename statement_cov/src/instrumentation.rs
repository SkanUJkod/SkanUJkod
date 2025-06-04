use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use cfg::ast::ParsedFile;
use cfg::cfg::ControlFlowGraph;
use go_parser::{
    ast::{Node, Stmt},
    AstObjects, FileSet, Token,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StatementMapping {
    pub stmt_id: usize,
    pub block_id: usize,
    pub stmt_index: usize,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub func_name: String,
    pub stmt_type: String,
}

#[derive(Debug)]
pub struct InstrumentationData {
    pub statement_mappings: HashMap<String, Vec<StatementMapping>>,
    pub total_statements_per_function: HashMap<String, usize>,
}

#[derive(Debug)]
struct SourceLocation {
    line: usize,
    column: usize,
}

pub fn generate_instrumented_project(
    cfgs: &HashMap<String, ControlFlowGraph>,
    parsed_files: &[ParsedFile],
    fset: &FileSet,
    objs: &AstObjects,
    source_path: &Path,
    output_path: &Path,
) -> Result<InstrumentationData> {
    fs::create_dir_all(output_path)?;

    copy_project_structure(source_path, output_path)?;

    let coverage_code = generate_coverage_tracker();
    let coverage_file = output_path.join("coverage_tracker.go");
    fs::write(&coverage_file, coverage_code)?;

    let mut instrumentation_data = InstrumentationData {
        statement_mappings: HashMap::new(),
        total_statements_per_function: HashMap::new(),
    };

    let mut file_to_functions: HashMap<PathBuf, Vec<String>> = HashMap::new();

    for (func_name, cfg) in cfgs {
        for (_, block) in &cfg.blocks {
            if let Some(stmt) = block.stmts.first() {
                let pos = stmt.stmt.pos(objs);
                if let Some(position) = fset.position(pos) {
                    let file_path = PathBuf::from(position.filename.as_ref());
                    file_to_functions
                        .entry(file_path)
                        .or_insert_with(Vec::new)
                        .push(func_name.clone());
                    break;
                }
            }
        }
    }

    for parsed_file in parsed_files {
        let relative_path = parsed_file
            .path
            .strip_prefix(source_path)
            .unwrap_or(&parsed_file.path);

        let output_file_path = output_path.join(relative_path);

        if let Some(parent) = output_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let functions_in_file = file_to_functions
            .get(&parsed_file.path)
            .cloned()
            .unwrap_or_default();

        let source_content = fs::read_to_string(&parsed_file.path)?;

        let instrumented_content = instrument_file_advanced(
            &source_content,
            &parsed_file.path,
            cfgs,
            fset,
            objs,
            &functions_in_file,
            &mut instrumentation_data,
        )?;

        fs::write(&output_file_path, instrumented_content)?;
    }

    let test_code = generate_test_runner(&instrumentation_data);
    let test_file = output_path.join("coverage_test.go");
    fs::write(&test_file, test_code)?;

    let mapping_json = serde_json::to_string_pretty(&instrumentation_data.statement_mappings)?;
    fs::write(output_path.join("stmt_mapping.json"), mapping_json)?;

    Ok(instrumentation_data)
}

fn instrument_file_advanced(
    source: &str,
    file_path: &Path,
    cfgs: &HashMap<String, ControlFlowGraph>,
    fset: &FileSet,
    objs: &AstObjects,
    functions_in_file: &[String],
    instrumentation_data: &mut InstrumentationData,
) -> Result<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut instrumented = String::new();
    let mut next_stmt_id = 0;

    let mut instrumentation_points: Vec<InstrumentationPoint> = Vec::new();

    for func_name in functions_in_file {
        if let Some(cfg) = cfgs.get(func_name) {
            let mut mappings = Vec::new();
            let mut total_stmts = 0;

            for (block_id, block) in &cfg.blocks {
                if *block_id == cfg.entry || block.stmts.is_empty() {
                    continue;
                }

                if block.succs.is_empty() && block.stmts.len() == 1 {
                    if let Stmt::Empty(e) = &block.stmts[0].stmt {
                        if e.implicit {
                            continue;
                        }
                    }
                }

                for (stmt_index, stmt) in block.stmts.iter().enumerate() {
                    if let Stmt::Empty(e) = &stmt.stmt {
                        if e.implicit {
                            continue;
                        }
                    }

                    let pos = stmt.stmt.pos(objs);
                    let end_pos = stmt.stmt.end(objs);

                    if let (Some(start_position), Some(end_position)) =
                        (fset.position(pos), fset.position(end_pos))
                    {
                        let file_path_str = file_path.to_str().unwrap_or("");
                        if start_position.filename.as_ref() == file_path_str {
                            let insert_point = find_instrumentation_point(
                                &stmt.stmt,
                                start_position.line,
                                start_position.column,
                                &lines,
                                objs,
                                fset,
                            );

                            instrumentation_points.push(InstrumentationPoint {
                                line: insert_point.line,
                                column: insert_point.column,
                                stmt_id: next_stmt_id,
                                func_name: func_name.clone(),
                                original_line: start_position.line,
                            });

                            mappings.push(StatementMapping {
                                stmt_id: next_stmt_id,
                                block_id: *block_id,
                                stmt_index,
                                line: start_position.line,
                                column: start_position.column,
                                end_line: end_position.line,
                                end_column: end_position.column,
                                func_name: func_name.clone(),
                                stmt_type: get_stmt_type(&stmt.stmt),
                            });

                            next_stmt_id += 1;
                            total_stmts += 1;
                        }
                    }
                }
            }

            if !mappings.is_empty() {
                instrumentation_data
                    .statement_mappings
                    .insert(func_name.clone(), mappings);
                instrumentation_data
                    .total_statements_per_function
                    .insert(func_name.clone(), total_stmts);
            }
        }
    }

    instrumentation_points.sort_by(|a, b| a.line.cmp(&b.line).then(a.column.cmp(&b.column)));

    apply_instrumentation(&lines, &instrumentation_points, &mut instrumented)?;

    Ok(instrumented)
}

#[derive(Debug)]
struct InstrumentationPoint {
    line: usize,
    column: usize,
    stmt_id: usize,
    func_name: String,
    original_line: usize,
}

fn find_instrumentation_point(
    stmt: &Stmt,
    start_line: usize,
    start_column: usize,
    lines: &[&str],
    objs: &AstObjects,
    fset: &FileSet,
) -> SourceLocation {
    match stmt {
        Stmt::Block(_) => SourceLocation {
            line: start_line,
            column: start_column,
        },
        Stmt::If(_) => SourceLocation {
            line: start_line,
            column: get_line_indent(lines, start_line - 1),
        },
        Stmt::Labeled(key) => {
            let label_stmt = &objs.l_stmts[*key];
            if let Some(pos) = fset.position(label_stmt.colon) {
                SourceLocation {
                    line: pos.line,
                    column: pos.column + 1,
                }
            } else {
                SourceLocation {
                    line: start_line,
                    column: start_column,
                }
            }
        }
        _ => SourceLocation {
            line: start_line,
            column: get_line_indent(lines, start_line - 1),
        },
    }
}

fn get_line_indent(lines: &[&str], line_idx: usize) -> usize {
    if line_idx < lines.len() {
        lines[line_idx]
            .chars()
            .take_while(|c| c.is_whitespace())
            .count()
    } else {
        0
    }
}

fn get_stmt_type(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Expr(_) => "expr",
        Stmt::Assign(_) => "assign",
        Stmt::If(_) => "if",
        Stmt::For(_) => "for",
        Stmt::Range(_) => "range",
        Stmt::Return(_) => "return",
        Stmt::Branch(b) => match b.token {
            Token::BREAK => "break",
            Token::CONTINUE => "continue",
            Token::GOTO => "goto",
            Token::FALLTHROUGH => "fallthrough",
            _ => "branch",
        },
        Stmt::Block(_) => "block",
        Stmt::Switch(_) => "switch",
        Stmt::TypeSwitch(_) => "typeswitch",
        Stmt::Case(_) => "case",
        Stmt::Labeled(_) => "labeled",
        Stmt::Go(_) => "go",
        Stmt::Defer(_) => "defer",
        Stmt::Send(_) => "send",
        Stmt::IncDec(_) => "incdec",
        Stmt::Select(_) => "select",
        Stmt::Decl(_) => "decl",
        Stmt::Empty(_) => "empty",
        _ => "unknown",
    }
    .to_string()
}

fn apply_instrumentation(
    lines: &[&str],
    points: &[InstrumentationPoint],
    output: &mut String,
) -> Result<()> {
    let mut current_col = 1;
    let mut point_idx = 0;

    for (line_idx, line_content) in lines.iter().enumerate() {
        let line_num = line_idx + 1;

        while point_idx < points.len() && points[point_idx].line == line_num {
            let point = &points[point_idx];

            if current_col == 1 && point.column > 0 {
                let indent = " ".repeat(point.column);
                output.push_str(&format!(
                    "{}stmt_hit(\"{}\", {}) // line {}\n",
                    indent, point.func_name, point.stmt_id, point.original_line
                ));
            }

            point_idx += 1;
        }

        output.push_str(line_content);
        output.push('\n');

        current_col = 1;
    }

    Ok(())
}

fn generate_coverage_tracker() -> String {
    r#"package main

import (
    "encoding/json"
    "fmt"
    "os"
    "sync"
    "runtime"
)

var (
    coverageMutex sync.Mutex
    coverageData  = make(map[string]map[int]bool)
    coverageInit  sync.Once
)

func initCoverage() {
    coverageInit.Do(func() {
        c := make(chan os.Signal, 1)
        go func() {
            for range c {
                saveCoverageData()
                os.Exit(0)
            }
        }()
        
        runtime.SetFinalizer(&coverageData, func(*map[string]map[int]bool) {
            saveCoverageData()
        })
    })
}

func stmt_hit(funcName string, stmtID int) {
    initCoverage()
    
    coverageMutex.Lock()
    defer coverageMutex.Unlock()
    
    if _, exists := coverageData[funcName]; !exists {
        coverageData[funcName] = make(map[int]bool)
    }
    coverageData[funcName][stmtID] = true
}

func saveCoverageData() {
    coverageMutex.Lock()
    defer coverageMutex.Unlock()
    
    exportCoverageData()
}

func exportCoverageData() error {
    exportData := make(map[string][]int)
    for funcName, stmts := range coverageData {
        var stmtList []int
        for stmtID := range stmts {
            stmtList = append(stmtList, stmtID)
        }
        exportData[funcName] = stmtList
    }
    
    data, err := json.MarshalIndent(exportData, "", "  ")
    if err != nil {
        return err
    }
    
    return os.WriteFile("coverage_data.json", data, 0644)
}

func printCoverage() {
    coverageMutex.Lock()
    defer coverageMutex.Unlock()
    
    fmt.Println("\n=== Statement Coverage Summary ===")
    totalFuncs := len(coverageData)
    totalStmts := 0
    
    for funcName, stmts := range coverageData {
        count := len(stmts)
        totalStmts += count
        fmt.Printf("  %s: %d statements executed\n", funcName, count)
    }
    
    fmt.Printf("\nTotal: %d statements in %d functions\n", totalStmts, totalFuncs)
}
"#
    .to_string()
}

fn generate_test_runner(instrumentation_data: &InstrumentationData) -> String {
    let mut code = String::new();
    code.push_str("package main\n\n");
    code.push_str("import (\n");
    code.push_str("    \"os\"\n");
    code.push_str("    \"testing\"\n");
    code.push_str("    \"time\"\n");
    code.push_str(")\n\n");

    code.push_str("func TestMain(m *testing.M) {\n");
    code.push_str("    initCoverage()\n");
    code.push_str("    \n");
    code.push_str("    code := m.Run()\n");
    code.push_str("    \n");
    code.push_str("    time.Sleep(100 * time.Millisecond)\n");
    code.push_str("    \n");
    code.push_str("    if err := exportCoverageData(); err != nil {\n");
    code.push_str("        panic(err)\n");
    code.push_str("    }\n");
    code.push_str("    \n");
    code.push_str("    printCoverage()\n");
    code.push_str("    \n");
    code.push_str("    os.Exit(code)\n");
    code.push_str("}\n\n");

    code.push_str("func TestCoverageEnabled(t *testing.T) {\n");
    code.push_str("    stmt_hit(\"TestCoverageEnabled\", 99999)\n");
    code.push_str("    \n");
    for func_name in instrumentation_data.total_statements_per_function.keys() {
        if !func_name.starts_with("Test") {
            code.push_str(&format!("    // {}()\n", func_name));
        }
    }
    code.push_str("}\n");

    code
}

fn copy_project_structure(source: &Path, dest: &Path) -> Result<()> {
    let go_mod = source.join("go.mod");
    if go_mod.exists() {
        let content = fs::read_to_string(&go_mod)?;
        fs::write(dest.join("go.mod"), content)?;
    } else {
        let module_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("instrumented");
        let go_mod_content = format!("module {}\n\ngo 1.19\n", module_name);
        fs::write(dest.join("go.mod"), go_mod_content)?;
    }

    let go_sum = source.join("go.sum");
    if go_sum.exists() {
        fs::copy(&go_sum, dest.join("go.sum"))?;
    }

    copy_non_go_files(source, dest)?;

    Ok(())
}

fn copy_non_go_files(src: &Path, dst: &Path) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if file_name_str.starts_with('.') {
            continue;
        }

        let dest_path = dst.join(&file_name);

        if path.is_dir() {
            if file_name_str != "vendor" && file_name_str != ".git" {
                fs::create_dir_all(&dest_path)?;
                copy_non_go_files(&path, &dest_path)?;
            }
        } else if path.is_file() {
            if file_name_str.ends_with("_test.go") || !file_name_str.ends_with(".go") {
                fs::copy(&path, &dest_path)?;
            }
        }
    }
    Ok(())
}

pub fn generate_instrumented_main(cfgs: &HashMap<String, ControlFlowGraph>) -> Result<String> {
    let mut code = String::new();
    code.push_str("package main\n\n");

    let mut global_stmt_id = 0;

    for (func_name, cfg) in cfgs {
        code.push_str(&format!("func {}() {{\n", func_name));

        let mut visited = std::collections::HashSet::new();
        track_blocks(
            &mut code,
            cfg,
            cfg.entry,
            &mut visited,
            func_name,
            &mut global_stmt_id,
        );

        code.push_str("}\n\n");
    }

    if !cfgs.contains_key("main") {
        code.push_str("func main() {}\n");
    }

    Ok(code)
}

fn track_blocks(
    code: &mut String,
    cfg: &ControlFlowGraph,
    block_id: usize,
    visited: &mut std::collections::HashSet<usize>,
    func_name: &str,
    stmt_counter: &mut usize,
) {
    if visited.contains(&block_id) {
        return;
    }
    visited.insert(block_id);

    if let Some(block) = cfg.blocks.get(&block_id) {
        for _stmt in &block.stmts {
            code.push_str(&format!(
                "    stmt_hit(\"{}\", {})\n",
                func_name, *stmt_counter
            ));
            *stmt_counter += 1;
        }

        for succ in &block.succs {
            track_blocks(code, cfg, *succ, visited, func_name, stmt_counter);
        }
    }
}

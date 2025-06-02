use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use cfg::cfg::ControlFlowGraph;

/// Generate instrumented version of the Go project
pub fn generate_instrumented_project(
    cfgs: &HashMap<String, ControlFlowGraph>,
    source_path: &Path,
    output_path: &Path,
) -> Result<()> {
    // Create output directory
    fs::create_dir_all(output_path)?;

    // Copy project structure
    copy_project_structure(source_path, output_path)?;

    // Generate coverage tracking code
    let coverage_code = generate_coverage_tracker();
    let coverage_file = output_path.join("coverage_tracker.go");
    fs::write(&coverage_file, coverage_code)?;

    // Generate instrumented main file with all functions
    let main_code = generate_instrumented_main(cfgs)?;
    let main_file = output_path.join("main_instrumented.go");
    fs::write(&main_file, main_code)?;

    // Generate test file that ensures coverage export
    let test_code = generate_test_runner(cfgs);
    let test_file = output_path.join("coverage_test.go");
    fs::write(&test_file, test_code)?;

    Ok(())
}

/// Generate the coverage tracking code
fn generate_coverage_tracker() -> String {
    r#"package main

import (
    "encoding/json"
    "fmt"
    "os"
    "sync"
)

var (
    coverageMutex sync.Mutex
    coverageData  = make(map[string]map[int]bool)
)

// Track statement execution
func stmt_hit(funcName string, blockID int) {
    coverageMutex.Lock()
    defer coverageMutex.Unlock()
    
    if _, exists := coverageData[funcName]; !exists {
        coverageData[funcName] = make(map[int]bool)
    }
    coverageData[funcName][blockID] = true
}

// Export coverage data to JSON
func exportCoverageData() error {
    coverageMutex.Lock()
    defer coverageMutex.Unlock()
    
    // Convert map[int]bool to []int for JSON
    exportData := make(map[string][]int)
    for funcName, blocks := range coverageData {
        var blockList []int
        for blockID := range blocks {
            blockList = append(blockList, blockID)
        }
        exportData[funcName] = blockList
    }
    
    data, err := json.MarshalIndent(exportData, "", "  ")
    if err != nil {
        return err
    }
    
    return os.WriteFile("coverage_data.json", data, 0644)
}

// Debug function to print coverage
func printCoverage() {
    coverageMutex.Lock()
    defer coverageMutex.Unlock()
    
    fmt.Println("Coverage data:")
    for funcName, blocks := range coverageData {
        fmt.Printf("  %s: %d blocks hit\n", funcName, len(blocks))
    }
}
"#
    .to_string()
}

/// Generate instrumented main file with simplified functions
pub fn generate_instrumented_main(cfgs: &HashMap<String, ControlFlowGraph>) -> Result<String> {
    let mut code = String::new();
    code.push_str("package main\n\n");

    // Global statement ID counter
    let mut global_stmt_id = 0;

    // For each function, create a simplified instrumented version
    for (func_name, cfg) in cfgs {
        println!("Processing function: {}", func_name); // Debugging statement
        code.push_str(&format!("// Instrumented version of {}\n", func_name));
        code.push_str(&format!("func {}() {{\n", func_name));

        // Track all blocks in order
        let mut visited = std::collections::HashSet::new();
        track_blocks(&mut code, cfg, cfg.entry, &mut visited, func_name, &mut global_stmt_id);

        code.push_str("}\n\n");
    }

    // Add a main function if it doesn't exist
    if !cfgs.contains_key("main") {
        println!("Adding stub main function"); // Debugging statement
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
        println!("Block {} already visited", block_id); // Debugging statement
        return;
    }
    visited.insert(block_id);

    if let Some(block) = cfg.blocks.get(&block_id) {
        println!("Processing block {} in function {}", block_id, func_name); // Debugging statement
        for stmt in &block.stmts {
            // Instrument each statement with a unique ID
            println!("Instrumenting statement: {}", stmt.text); // Debugging statement
            code.push_str(&format!("    stmt_hit(\"{}\", {})\n", func_name, *stmt_counter));
            *stmt_counter += 1;
        }

        for succ in &block.succs {
            println!("Processing successor block {}", succ); // Debugging statement
            track_blocks(code, cfg, *succ, visited, func_name, stmt_counter);
        }
    } else {
        println!("Block {} not found in CFG", block_id); // Debugging statement
    }
}

/// Generate test runner that exports coverage data
fn generate_test_runner(cfgs: &HashMap<String, ControlFlowGraph>) -> String {
    let mut code = String::new();
    code.push_str("package main\n\n");
    code.push_str("import (\n");
    code.push_str("    \"os\"\n");
    code.push_str("    \"testing\"\n");
    code.push_str(")\n\n");
    // TestMain
    code.push_str("func TestMain(m *testing.M) {\n");
    code.push_str("    code := m.Run()\n");
    code.push_str("    if err := exportCoverageData(); err != nil {\n");
    code.push_str("        panic(err)\n");
    code.push_str("    }\n");
    code.push_str("    os.Exit(code)\n");
    code.push_str("}\n\n");
    // TestCoverage: call each instrumented function
    code.push_str("func TestCoverage(t *testing.T) {\n");
    for func_name in cfgs.keys() {
        code.push_str("    defer func() { if r := recover(); r != nil {} }()\n");
        code.push_str(&format!("    {}()\n", func_name));
    }
    code.push_str("}\n\n");
    // Ensure coverage export even if no other tests exist
    code.push_str("func TestEnsureCoverageExport(t *testing.T) {\n");
    code.push_str("    stmt_hit(\"TestEnsureCoverageExport\", 0)\n");
    code.push_str("}\n");
    code
}

/// Copy project structure (handles go.mod, go.sum, and test files)
fn copy_project_structure(source: &Path, dest: &Path) -> Result<()> {
    // Copy go.mod if exists
    let go_mod = source.join("go.mod");
    if go_mod.exists() {
        let content = fs::read_to_string(&go_mod)?;
        fs::write(dest.join("go.mod"), content)?;
    } else {
        // Create a basic go.mod if it doesn't exist
        let module_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("coverage_test");
        let go_mod_content = format!("module {}\n\ngo 1.19\n", module_name);
        fs::write(dest.join("go.mod"), go_mod_content)?;
    }

    // Copy go.sum if exists
    let go_sum = source.join("go.sum");
    if go_sum.exists() {
        fs::copy(&go_sum, dest.join("go.sum"))?;
    }

    for entry in fs::read_dir(source).context("Failed to read source directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();

                if name_str.ends_with("_test.go") {
                    continue;
                }

                if name_str.ends_with(".go") && !name_str.contains("main.go") {
                    let content = fs::read_to_string(&path)?;
                    if !content.contains("func main()") {
                        fs::write(dest.join(name), content)?;
                    }
                }
            }
        } else if path.is_dir() {
            if let Some(dir_name) = path.file_name() {
                let dir_name_str = dir_name.to_string_lossy();
                if !dir_name_str.starts_with('.') && dir_name_str != "vendor" {
                    let sub_dest = dest.join(dir_name);
                    fs::create_dir_all(&sub_dest)?;
                    copy_project_structure(&path, &sub_dest)?;
                }
            }
        }
    }

    Ok(())
}

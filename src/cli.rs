use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use anyhow::{Result, Context};
use serde_json;

use statement_cov::ProjectCoverage as StmtCoverage;
use cyclomatic_complexity::ProjectComplexity;
use branch_cov::ProjectBranchCoverage;

#[derive(Clone)]
pub enum ImageFormat {
    PNG,
    SVG,
    PDF
}

impl ImageFormat {
    pub fn from_str(format: &str) -> Option<Self> {
        match format.to_lowercase().as_str() {
            "png" => Some(Self::PNG),
            "svg" => Some(Self::SVG),
            "pdf" => Some(Self::PDF),
            _ => None
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::PNG => "png",
            Self::SVG => "svg",
            Self::PDF => "pdf",
        }
    }
}

pub fn check_graphviz_installation() -> Result<bool> {
    let output = Command::new("dot")
        .arg("-V")
        .output();

    match output {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false)
    }
}

pub fn generate_image_from_dot(
    dot_content: &str, 
    output_path: &Path, 
    format: ImageFormat
) -> Result<()> {
    if !check_graphviz_installation()? {
        anyhow::bail!("Graphviz is not installed. Please install 'graphviz' package to generate images.");
    }

    let temp_dot_path = output_path.with_extension("dot");
    fs::write(&temp_dot_path, dot_content)?;

    let status = Command::new("dot")
        .arg("-T")
        .arg(format.extension())
        .arg("-o")
        .arg(output_path)
        .arg(&temp_dot_path)
        .status()
        .context("Failed to run 'dot' command")?;

    if !status.success() {
        anyhow::bail!("The 'dot' command failed to execute");
    }

    if temp_dot_path != output_path {
        let _ = fs::remove_file(temp_dot_path);
    }

    Ok(())
}

pub fn print_usage() {
    eprintln!(
        "Usage:\n  \
        skan-uj-kod <command> [options] <path>\n\n\
        Commands:\n  \
        cfg                   - Display CFG in debug format\n  \
        dot <FuncName>        - Print DOT format for selected function\n  \
        dot-all               - Generate DOT files for all functions\n  \
        stmt-cov              - Display statement coverage\n  \
        branch-cov            - Display branch coverage analysis\n  \
        complexity            - Display cyclomatic complexity analysis\n  \
        help                  - Show this help message\n\n\
        Options:\n  \
        --json                - Output in JSON format\n  \
        --output=<file>       - Write output to file\n  \
        --image=<format>      - Generate image (png, svg, pdf) from DOT\n  \
        --verbose             - Show detailed information"
    );
}

pub fn print_complexity_report(complexity: &ProjectComplexity, detailed: bool) {
    println!("\n=== Cyclomatic Complexity Report ===\n");
    println!("Files analyzed: {}", complexity.files_analyzed);
    println!("Functions found: {}", complexity.total_functions);
    println!("Average complexity: {:.2}", complexity.average_complexity);
    println!("Maximum complexity: {} ({})\n", 
            complexity.max_complexity, complexity.max_complexity_function);

    println!("Complexity Distribution:");
    println!("  Low (1-5): {} functions", complexity.complexity_distribution.get("low").unwrap_or(&0));
    println!("  Moderate (6-10): {} functions", complexity.complexity_distribution.get("moderate").unwrap_or(&0));
    println!("  High (11-20): {} functions", complexity.complexity_distribution.get("high").unwrap_or(&0));
    println!("  Very High (>20): {} functions\n", complexity.complexity_distribution.get("very_high").unwrap_or(&0));

    if detailed {
        println!("Function Details:");
        let mut sorted_functions: Vec<_> = complexity.functions.iter().collect();
        sorted_functions.sort_by(|a, b| b.1.cyclomatic_complexity.cmp(&a.1.cyclomatic_complexity));
        
        for (name, func) in sorted_functions.iter().take(10) {
            println!("{} - CC: {}, Cognitive: {}, LOC: {}",
                name, 
                func.cyclomatic_complexity,
                func.cognitive_complexity,
                func.lines_of_code
            );
        }
    }
}

pub fn print_coverage_report(coverage: &StmtCoverage, detailed: bool) {
    println!("\n=== Statement Coverage Report ===\n");
    println!("Files analyzed: {}", coverage.files_analyzed);
    println!("Functions found: {}", coverage.functions.len());
    println!("Total statements: {}", coverage.total_statements);
    println!("Covered statements: {}", coverage.covered_statements);
    println!("Overall coverage: {:.1}%\n", coverage.overall_coverage);

    if detailed {
        println!("Function Details:");
        let mut functions: Vec<_> = coverage.functions.iter().collect();
        functions.sort_by(|a, b| a.1.coverage_percentage.partial_cmp(&b.1.coverage_percentage).unwrap());
        
        for (func_name, func_coverage) in functions {
            println!("{}", func_name);
            println!("  Coverage: {:.1}% ({}/{})",
                func_coverage.coverage_percentage,
                func_coverage.covered_statements,
                func_coverage.total_statements
            );
            
            if !func_coverage.uncovered_lines.is_empty() && func_coverage.uncovered_lines.len() <= 5 {
                println!("  Uncovered lines: {:?}", func_coverage.uncovered_lines);
            } else if !func_coverage.uncovered_lines.is_empty() {
                println!("  Uncovered lines: {} (showing first 5: {:?}...)", 
                    func_coverage.uncovered_lines.len(),
                    &func_coverage.uncovered_lines[..5.min(func_coverage.uncovered_lines.len())]
                );
            }
            println!();
        }
    }
}

pub fn print_branch_coverage_report(coverage: &ProjectBranchCoverage, detailed: bool) {
    println!("\n=== Branch Coverage Report ===\n");
    println!("Files analyzed: {}", coverage.files_analyzed.len());
    println!("Functions found: {}", coverage.functions.len());
    println!("Total branches: {}", coverage.total_branches);
    println!("Covered branches: {}", coverage.covered_branches);
    println!("Overall coverage: {:.1}%\n", coverage.overall_coverage_percentage);

    if detailed {
        println!("Function Details:");
        let mut functions: Vec<_> = coverage.functions.iter().collect();
        functions.sort_by(|a, b| a.1.coverage_percentage.partial_cmp(&b.1.coverage_percentage).unwrap());
        
        for (func_name, func_coverage) in functions {
            println!("{}", func_name);
            println!("  Coverage: {:.1}% ({}/{})",
                func_coverage.coverage_percentage,
                func_coverage.covered_branches,
                func_coverage.total_branches
            );
            
            if !func_coverage.uncovered_branches.is_empty() && func_coverage.uncovered_branches.len() <= 5 {
                println!("  Uncovered branches:");
                for uncovered in &func_coverage.uncovered_branches[..5.min(func_coverage.uncovered_branches.len())] {
                    println!("    Line {}: {} ({})", uncovered.line, uncovered.branch_type, uncovered.condition);
                }
            } else if !func_coverage.uncovered_branches.is_empty() {
                println!("  Uncovered branches: {} (showing first 5)", func_coverage.uncovered_branches.len());
                for uncovered in &func_coverage.uncovered_branches[..5] {
                    println!("    Line {}: {} ({})", uncovered.line, uncovered.branch_type, uncovered.condition);
                }
                println!("    ...");
            }
            println!();
        }

        if !coverage.uncovered_branches.is_empty() {
            println!("Overall Uncovered Branches Summary:");
            println!("Total uncovered: {}", coverage.uncovered_branches.len());
        }
    }
}

pub fn export_to_json<T: serde::Serialize>(data: &T, output: Option<&Path>) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    
    if let Some(path) = output {
        fs::write(path, json).context("Failed to write JSON to file")?;
        println!("Output written to: {}", path.display());
    } else {
        println!("{}", json);
    }
    
    Ok(())
}
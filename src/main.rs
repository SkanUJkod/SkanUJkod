use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use std::collections::HashMap;
use std::process::Command;
use anyhow::{Result, Context};
use serde_json;

use cfg::{parse_project, build_cfgs_for_file, to_dot};
use statement_cov::{
    analyze_statement_coverage_with_options,
    CoverageOptions,
    ProjectCoverage as StmtCoverage
};
use cyclomatic_complexity::{
    analyze_cyclomatic_complexity_with_options,
    ComplexityOptions,
    ProjectComplexity
};

mod cli {
    use super::*;

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
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        cli::print_usage();
        std::process::exit(1);
    }

    let command = &args[1];
    
    if command == "help" {
        cli::print_usage();
        return Ok(());
    }
    
    let json_output = args.iter().any(|arg| arg == "--json");
    let verbose = args.iter().any(|arg| arg == "--verbose");
    
    let image_format = args.iter()
        .find(|arg| arg.starts_with("--image="))
        .and_then(|arg| {
            let format = &arg[8..];
            cli::ImageFormat::from_str(format)
        });
    
    let output_path = args.iter()
        .filter_map(|arg| {
            if arg.starts_with("--output=") {
                Some(arg[9..].to_string())
            } else {
                None
            }
        })
        .next()
        .map(PathBuf::from);
    
    let project_path = match args.last() {
        Some(path) if !path.starts_with("--") => Path::new(path),
        _ => {
            eprintln!("Error: No project path provided");
            cli::print_usage();
            std::process::exit(1);
        }
    };
    
    if !project_path.exists() {
        eprintln!("Error: Project path does not exist: {}", project_path.display());
        std::process::exit(1);
    }
    
    match command.as_str() {
        "cfg" => {
            let (fset, objs, files) = parse_project(project_path)?;
            let mut cfgs_map = HashMap::new();
            
            for pf in &files {
                let per_file_map = build_cfgs_for_file(&fset, &objs, &pf.ast);
                cfgs_map.extend(per_file_map);
            }
            
            if json_output {
                let mut cfg_data = HashMap::new();
                for (name, cfg) in &cfgs_map {
                    cfg_data.insert(name.clone(), format!("{:#?}", cfg));
                }
                
                cli::export_to_json(&cfg_data, output_path.as_deref())?;
            } else {
                println!("=== Control Flow Graphs ===");
                for (fname, graph) in &cfgs_map {
                    println!("--- func {} ---\n{:#?}", fname, graph);
                }
            }
        },
        
        "dot" => {
            if args.len() < 4 {
                eprintln!("Error: Missing function name for 'dot' command");
                cli::print_usage();
                std::process::exit(1);
            }
            
            let func_name = &args[2];
            let (fset, objs, files) = parse_project(project_path)?;
            
            let mut cfgs_map = HashMap::new();
            for pf in &files {
                let per_file_map = build_cfgs_for_file(&fset, &objs, &pf.ast);
                cfgs_map.extend(per_file_map);
            }
            
            if let Some(graph) = cfgs_map.get(func_name) {
                let dot = to_dot(graph, func_name);
                
                if let Some(format) = image_format {
                    let output = output_path.unwrap_or_else(|| 
                        PathBuf::from(format!("{}.{}", func_name, format.extension()))
                    );
                    
                    cli::generate_image_from_dot(&dot, &output, format)?;
                    println!("Graph image for '{}' generated: {}", func_name, output.display());
                } else if let Some(path) = output_path {
                    fs::write(&path, dot)?;
                    println!("DOT graph for '{}' written to: {}", func_name, path.display());
                } else {
                    println!("{}", dot);
                }
            } else {
                eprintln!("Error: Function '{}' not found", func_name);
                std::process::exit(1);
            }
        },
        
        "dot-all" => {
            let (fset, objs, files) = parse_project(project_path)?;
            
            let mut cfgs_map = HashMap::new();
            for pf in &files {
                let per_file_map = build_cfgs_for_file(&fset, &objs, &pf.ast);
                cfgs_map.extend(per_file_map);
            }
            
            let out_dir = output_path.unwrap_or(PathBuf::from("dot_output"));
            if !out_dir.exists() {
                fs::create_dir_all(&out_dir)?;
            }
            
            for (fname, graph) in cfgs_map {
                let dot = to_dot(&graph, &fname);
                
                if let Some(format) = &image_format {
                    let filepath = out_dir.join(format!("{}.{}", fname, format.extension()));
                    cli::generate_image_from_dot(&dot, &filepath, format.clone())?;
                    if verbose {
                        println!("Generated image: {}", filepath.display());
                    }
                } else {
                    let filepath = out_dir.join(format!("{}.dot", fname));
                    fs::write(&filepath, dot)?;
                    if verbose {
                        println!("Generated DOT: {}", filepath.display());
                    }
                }
            }
            
            if let Some(format) = &image_format {
                println!("Graph images generated in directory: {} (format: {})", out_dir.display(), format.extension());
            } else {
                println!("DOT graphs generated in directory: {}", out_dir.display());
            }
        },
        
        "stmt-cov" => {
            let options = CoverageOptions {
                verbose,
                ..CoverageOptions::default()
            };
            
            let coverage = analyze_statement_coverage_with_options(project_path, &options)?;
            
            if json_output {
                cli::export_to_json(&coverage, output_path.as_deref())?;
            } else {
                cli::print_coverage_report(&coverage, verbose);
            }
        },
        
        "complexity" => {
            let options = ComplexityOptions {
                verbose,
                ..ComplexityOptions::default()
            };
            
            let complexity = analyze_cyclomatic_complexity_with_options(project_path, &options)?;
            
            if json_output {
                cli::export_to_json(&complexity, output_path.as_deref())?;
            } else {
                cli::print_complexity_report(&complexity, verbose);
            }
        },
        
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            cli::print_usage();
            std::process::exit(1);
        }
    }
    
    Ok(())
}
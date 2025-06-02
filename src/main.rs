use cfg::ast::parse_project;
use cfg::cfg::build_cfgs_for_file;
use cfg::export::to_dot;
use statement_cov::{
    analyze_statement_coverage_with_options,
    print_coverage_report,
    export_coverage_report,
    CoverageOptions,
    ExportFormat,
};

use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::Command;

fn print_help() {
    println!("Statement Coverage Tool for Go Projects\n");
    println!("Usage:");
    println!("  skan-uj-kod <path>                      - Print CFG in Debug format");
    println!("  skan-uj-kod --dot <FuncName> <path>     - Print DOT format for selected function");
    println!("  skan-uj-kod --dot-all <path>            - Print DOT for all functions");
    println!("  skan-uj-kod --stmt-cov <path>           - Print overall statement coverage percentage");
    println!("  skan-uj-kod --full-cov <path>           - Print per-function coverage details");
    println!("  skan-uj-kod --analyze <path> [options]  - Advanced coverage analysis");
    println!("\nAdvanced options for --analyze:");
    println!("  --verbose                               - Show detailed output");
    println!("  --export <format> <file>                - Export report (json|html|markdown)");
    println!("  --timeout <seconds>                     - Test timeout (default: 300)");
    println!("  --test-args <args>                      - Additional go test arguments");
    println!("\nExamples:");
    println!("  skan-uj-kod --analyze ./my-project --verbose");
    println!("  skan-uj-kod --analyze ./my-project --export html coverage.html");
    println!("  skan-uj-kod --analyze ./my-project --test-args \"-run TestSpecific\"");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }

    // Handle help
    if args[1] == "--help" || args[1] == "-h" {
        print_help();
        return Ok(());
    }

    // Advanced analysis mode
    if args[1] == "--analyze" {
        if args.len() < 3 {
            eprintln!("Error: --analyze requires a path");
            std::process::exit(1);
        }
        
        let path = Path::new(&args[2]);
        let mut options = CoverageOptions::default();
        let mut export_format = None;
        let mut export_path = None;
        
        // Parse additional options
        let mut i = 3;
        while i < args.len() {
            match args[i].as_str() {
                "--verbose" => {
                    options.verbose = true;
                    i += 1;
                }
                "--export" => {
                    if i + 2 >= args.len() {
                        eprintln!("Error: --export requires format and filename");
                        std::process::exit(1);
                    }
                    export_format = match args[i + 1].as_str() {
                        "json" => Some(ExportFormat::Json),
                        "html" => Some(ExportFormat::Html),
                        "markdown" | "md" => Some(ExportFormat::Markdown),
                        _ => {
                            eprintln!("Error: Unknown export format: {}", args[i + 1]);
                            std::process::exit(1);
                        }
                    };
                    export_path = Some(args[i + 2].clone());
                    i += 3;
                }
                "--timeout" => {
                    if i + 1 >= args.len() {
                        eprintln!("Error: --timeout requires a value");
                        std::process::exit(1);
                    }
                    options.timeout_seconds = args[i + 1].parse().unwrap_or(300);
                    i += 2;
                }
                "--test-args" => {
                    if i + 1 >= args.len() {
                        eprintln!("Error: --test-args requires a value");
                        std::process::exit(1);
                    }
                    options.test_args = args[i + 1].split_whitespace()
                        .map(String::from)
                        .collect();
                    i += 2;
                }
                _ => {
                    eprintln!("Error: Unknown option: {}", args[i]);
                    std::process::exit(1);
                }
            }
        }
        
        // Always show some output
        if !options.verbose {
            println!("🔍 Analyzing coverage for: {}", path.display());
        }
        
        match analyze_statement_coverage_with_options(path, &options) {
            Ok(coverage) => {
                // Always print report unless exporting
                if export_format.is_none() || options.verbose {
                    print_coverage_report(&coverage);
                }
                
                // Export if requested
                if let (Some(format), Some(output)) = (export_format, export_path) {
                    let output_path = Path::new(&output);
                    export_coverage_report(&coverage, output_path, format)?;
                    println!("✅ Report exported to: {}", output);
                }
                
                // Exit with non-zero if coverage is low
                if coverage.overall_coverage < 80.0 {
                    std::process::exit(2);
                }
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                std::process::exit(1);
            }
        }
        
        return Ok(());
    }

    // Legacy modes for backward compatibility
    match args[1].as_str() {
        "--full-cov" => {
            if args.len() != 3 {
                eprintln!("Error: --full-cov requires exactly one path argument");
                std::process::exit(1);
            }
            let options = CoverageOptions {
                verbose: true,
                ..Default::default()
            };
            let coverage = analyze_statement_coverage_with_options(Path::new(&args[2]), &options)?;
            // Already printed by verbose mode
        }
        
        "--stmt-cov" => {
            if args.len() != 3 {
                eprintln!("Error: --stmt-cov requires exactly one path argument");
                std::process::exit(1);
            }
            let coverage = analyze_statement_coverage_with_options(
                Path::new(&args[2]),
                &CoverageOptions::default()
            )?;
            println!("{:.2}%", coverage.overall_coverage);
        }
        
        "--dot" => {
            if args.len() < 4 {
                eprintln!("Error: --dot requires function name and path");
                std::process::exit(1);
            }
            let func_name = &args[2];
            let root_path = Path::new(&args[3]);
            
            let (fset, objs, files) = parse_project(root_path)?;
            let mut cfgs_map = HashMap::new();
            
            for pf in &files {
                let per_file_map = build_cfgs_for_file(&fset, &objs, &pf.ast);
                cfgs_map.extend(per_file_map);
            }
            
            if let Some(graph) = cfgs_map.get(func_name) {
                let dot = to_dot(graph, func_name);
                println!("{}", dot);
            } else {
                eprintln!("Function '{}' not found", func_name);
                std::process::exit(1);
            }
        }
        
        "--dot-all" => {
            if args.len() != 3 {
                eprintln!("Error: --dot-all requires exactly one path argument");
                std::process::exit(1);
            }
            let root_path = Path::new(&args[2]);
            let out_dir = Path::new("out");
            
            if !out_dir.exists() {
                std::fs::create_dir_all(out_dir)?;
            }

            let img_dir = out_dir.join("images");
            if !img_dir.exists() {
                std::fs::create_dir_all(&img_dir)?;
            }
            
            let (fset, objs, files) = parse_project(root_path)?;
            let mut cfgs_map = HashMap::new();
            
            for pf in &files {
                let per_file_map = build_cfgs_for_file(&fset, &objs, &pf.ast);
                cfgs_map.extend(per_file_map);
            }
            
            for (fname, graph) in cfgs_map {
                let dot_content = to_dot(&graph, &fname);
                let dot_path = out_dir.join(format!("{}.dot", fname));
                std::fs::write(&dot_path, &dot_content)?;

                // Generate PNG image from the .dot file
                let img_path = img_dir.join(format!("{}.png", fname));
                let status = Command::new("dot")
                    .arg("-Tpng")
                    .arg(&dot_path)
                    .arg("-o")
                    .arg(&img_path)
                    .status()?;

                if !status.success() {
                    eprintln!("Warning: Failed to generate image for {}", fname);
                }
            }
            
            println!("✅ DOT files exported to: {}", out_dir.display());
            println!("✅ PNG images generated in: {}", img_dir.display());
        }
        
        _ => {
            // Default mode - print CFG debug
            let root_path = Path::new(&args[1]);
            let (fset, objs, files) = parse_project(root_path)?;
            
            println!("=== Control Flow Graphs ===");
            for pf in &files {
                let cfgs = build_cfgs_for_file(&fset, &objs, &pf.ast);
                for (fname, graph) in &cfgs {
                    println!("\n--- Function: {} ---", fname);
                    println!("{:#?}", graph);
                }
            }
        }
    }

    Ok(())
}
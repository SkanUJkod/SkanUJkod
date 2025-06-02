use cfg::ast::parse_project;
use cfg::cfg::build_cfgs_for_file;
use cfg::export::to_dot;
use statement_cov::{
    analyze_statement_coverage_with_options,
    CoverageOptions,
};

use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::path::Path;

fn print_help() {
    println!("Statement Coverage Tool for Go Projects\n");
    println!("Usage:");
    println!("  skan-uj-kod <path>                      - Print CFG in Debug format");
    println!("  skan-uj-kod --dot <FuncName> <path>     - Print DOT format for selected function");
    println!("  skan-uj-kod --dot-all <path>            - Print DOT for all functions");
    println!("  skan-uj-kod --stmt-cov <path>           - Print overall statement coverage percentage");
    println!("\nExamples:");
    println!("  skan-uj-kod --stmt-cov ./my-project");
    println!("  skan-uj-kod --dot-all ./my-project");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }

    if args[1] == "--help" || args[1] == "-h" {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "--stmt-cov" => {
            if args.len() != 3 {
                eprintln!("Error: --stmt-cov requires exactly one path argument");
                std::process::exit(1);
            }
            let coverage = analyze_statement_coverage_with_options(Path::new(&args[2]), &CoverageOptions::default())?;
            println!("{:.2}%", coverage.overall_coverage);
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

            let (fset, objs, files) = parse_project(root_path)?;
            let mut cfgs_map = HashMap::new();

            for pf in &files {
                let per_file_map = build_cfgs_for_file(&fset, &objs, &pf.ast);
                cfgs_map.extend(per_file_map);
            }

            for (fname, graph) in cfgs_map {
                let dot = to_dot(&graph, &fname);
                let filepath = out_dir.join(format!("{}.dot", fname));
                std::fs::write(filepath, dot)?;
            }

            println!("✅ DOT files exported to: {}", out_dir.display());
        }
        _ => {
            print_help();
            std::process::exit(1);
        }
    }

    Ok(())
}
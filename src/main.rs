use cfg::ast::parse_project;
use cfg::cfg::build_cfgs_for_file;
use cfg::export::to_dot;
use statement_cov::helpers::{CoverageAnalyzer, CoverageConfig};

use anyhow::Result;
use std::collections::HashMap;
use std::env;

fn main() -> Result<()> {
    // Minimal CLI:
    //   skan-uj-kod <path>                -> print CFG in Debug format
    //   skan-uj-kod --dot <FuncName> <path> -> print DOT format for selected function
    //   skan-uj-kod --dot-all <path>       -> print DOT for all functions
    //   skan-uj-kod --stmt-cov <path>      -> print overall statement coverage percentage
    //   skan-uj-kod --full-cov <path>      -> print per-function coverage

    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        eprintln!(
            "Usage:\n  skan-uj-kod <path>\n  skan-uj-kod --dot <FuncName> <path>\n  skan-uj-kod --dot-all <path>\n  skan-uj-kod --stmt-cov <path>\n  skan-uj-kod --full-cov <path>"
        );
        std::process::exit(1);
    }

    // Full per-function coverage
    if args[0] == "--full-cov" {
        if args.len() != 2 {
            eprintln!("Need: --full-cov <path>");
            std::process::exit(1);
        }
        let config = CoverageConfig::default();
        let analyzer = CoverageAnalyzer::new(config);
        let coverage = analyzer.analyze(std::path::Path::new(&args[1]))?;
        for (func, func_cov) in &coverage.functions {
            println!(
                "Function `{}`: total={} covered={} coverage={:.2}%",
                func,
                func_cov.total_statements,
                func_cov.covered_statements,
                func_cov.coverage_percentage
            );
        }
        return Ok(());
    }

    // Overall statement coverage
    if args[0] == "--stmt-cov" {
        if args.len() != 2 {
            eprintln!("Need: --stmt-cov <path>");
            std::process::exit(1);
        }
        let root = &args[1];
        let config = CoverageConfig::default();
        let analyzer = CoverageAnalyzer::new(config);
        let coverage = analyzer.analyze(std::path::Path::new(root))?;
        println!("{:.2}%", coverage.overall_coverage);
        return Ok(());
    }

    let mut dot_mode: Option<String> = None;
    let mut dot_all_mode = false;

    // DOT export for a single function
    if args[0] == "--dot" {
        if args.len() < 3 {
            eprintln!("Need: --dot <FuncName> <path>");
            std::process::exit(1);
        }
        dot_mode = Some(args[1].clone());
        args.remove(0);
        args.remove(0);
    }
    // DOT export for all functions
    else if args[0] == "--dot-all" {
        if args.len() != 2 {
            eprintln!("Need: --dot-all <path>");
            std::process::exit(1);
        }
        dot_all_mode = true;
        args.remove(0);
    }

    let root = &args[0];
    let root_path = std::path::Path::new(root);

    let (fset, objs, files) = parse_project(root_path)?;

    if dot_all_mode {
        let out_dir = std::path::Path::new("out");
        if !out_dir.exists() {
            std::fs::create_dir_all(out_dir)?;
        }
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
        return Ok(());
    }

    let mut cfgs_map = HashMap::new();
    for pf in &files {
        let per_file_map = build_cfgs_for_file(&fset, &objs, &pf.ast);
        cfgs_map.extend(per_file_map);
    }

    match &dot_mode {
        Some(func_name) => {
            if let Some(graph) = cfgs_map.get(func_name) {
                let dot = to_dot(graph, func_name);
                println!("{}", dot);
                return Ok(());
            } else {
                eprintln!("Function not found for DOT export");
                std::process::exit(2);
            }
        }
        None => {
            println!("=== CFGs ===");
            for (fname, graph) in &cfgs_map {
                println!("--- func {} ---\n{:#?}", fname, graph);
            }
        }
    }

    Ok(())
}

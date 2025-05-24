mod ast;
mod cfg;
mod dot;

#[cfg(test)]
mod cfg_tests;

use anyhow::Result;
use std::env;

fn main() -> Result<()> {
    // -----------------------------------------------
    // Minimal CLI:
    //   skan-uj-kod <path>                -> print CFG in Debug format
    //   skan-uj-kod --dot <FuncName> <path> -> print DOT format for selected function
    // -----------------------------------------------
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        eprintln!(
            "Usage:\n  skan-uj-kod <path>\n  skan-uj-kod --dot <FuncName> <path>\n  skan-uj-kod --dot-all <path>"
        );
        std::process::exit(1);
    }

    let mut dot_mode: Option<String> = None;
    let mut dot_all_mode = false;

    if args.get(0).map(|s| s == "--dot").unwrap_or(false) {
        if args.len() < 3 {
            eprintln!("Need: --dot <FuncName> <path>");
            std::process::exit(1);
        }
        dot_mode = Some(args[1].clone());
        args.remove(0);
        args.remove(0);
    } else if args.get(0).map(|s| s == "--dot-all").unwrap_or(false) {
        if args.len() < 2 {
            eprintln!("Need: --dot-all <path>");
            std::process::exit(1);
        }
        dot_all_mode = true;
        args.remove(0);
    }

    let root = &args[0];
    let root_path = std::path::Path::new(root);

    let (fset, objs, files) = ast::parse_project(root_path)?;

    if dot_all_mode {
        let out_dir = std::path::Path::new("out");
        if !out_dir.exists() {
            std::fs::create_dir_all(out_dir)?;
        }
        for parsed in files {
            let cfgs = cfg::build_cfgs_for_file(&fset, &objs, &parsed.ast);
            for (fname, graph) in cfgs {
                let dot = dot::to_dot(&graph, &fname);
                let filepath = out_dir.join(format!("{}.dot", fname));
                std::fs::write(filepath, dot)?;
            }
        }
        return Ok(());
    }

    for parsed in files {
        let cfgs = cfg::build_cfgs_for_file(&fset, &objs, &parsed.ast);

        match &dot_mode {
            Some(func_name) => {
                if let Some(graph) = cfgs.get(func_name) {
                    let dot = dot::to_dot(graph, func_name);
                    println!("{}", dot);
                    return Ok(());
                }
            }
            None => {
                println!("=== {} ===", parsed.path.display());
                for (fname, graph) in cfgs {
                    println!("--- func {} ---\n{:#?}", fname, graph);
                }
            }
        }
    }

    if dot_mode.is_some() {
        eprintln!("Function not found for DOT export");
        std::process::exit(2);
    }

    Ok(())
}
use crate::cfg::types::ControlFlowGraph;
use go_parser::ast::Decl;
use go_parser::{AstObjects, FileSet};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

/// Build CFGs for all functions in a file
pub fn build_cfgs_for_file(
    fset: &FileSet,
    objs: &AstObjects,
    file: &go_parser::ast::File,
) -> HashMap<String, ControlFlowGraph> {
    let mut out = HashMap::new();
    for decl in &file.decls {
        if let Decl::Func(fk) = *decl {
            let fdecl = &objs.fdecls[fk];
            let name = objs.idents[fdecl.name].name.clone();
            out.insert(name, ControlFlowGraph::build(fset, fdecl, objs));
        }
    }
    out
}

/// Export instrumented Go code for a function's CFG
pub fn export_instrumented_go(
    func_name: &str,
    cfg: &ControlFlowGraph,
    output_path: &std::path::Path,
) -> std::io::Result<()> {
    let mut file = File::create(output_path)?;
    writeln!(file, "package main\n")?;
    writeln!(file, "var hits = map[string]map[int]bool{{}}")?;
    writeln!(
        file,
        "func stmt_hit(funcName string, id int) {{\n\tif _, ok := hits[funcName]; !ok {{\n\t\thits[funcName] = map[int]bool{{}}\n\t}}\n\thits[funcName][id] = true\n}}\n"
    )?;

    writeln!(file, "func {}() {{", func_name)?;

    for (block_id, block) in &cfg.blocks {
        for _stmt in &block.stmts {
            writeln!(file, "\tstmt_hit(\"{}\", {});", func_name, block_id)?;
        }
    }

    writeln!(file, "}}")?;
    Ok(())
}

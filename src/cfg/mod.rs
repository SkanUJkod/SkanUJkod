// cfg/mod.rs
pub mod basic_block;
pub mod builder;
pub mod graph;
pub mod optimization;
pub mod stmt_handlers;
pub mod validation;

pub use basic_block::BasicBlock;
pub use builder::CfgBuilder;
pub use graph::ControlFlowGraph;

use go_parser::ast::FuncDecl;
use go_parser::{AstObjects, FileSet};
use std::collections::HashMap;

/// Build CFGs for all functions in a file
pub fn build_cfgs_for_file(
    fset: &FileSet,
    objs: &AstObjects,
    file: &go_parser::ast::File,
) -> HashMap<String, ControlFlowGraph> {
    let mut out = HashMap::new();

    for decl in &file.decls {
        if let go_parser::ast::Decl::Func(fk) = *decl {
            let fdecl = &objs.fdecls[fk];
            let name = objs.idents[fdecl.name].name.clone();
            out.insert(name, ControlFlowGraph::build(fset, fdecl, objs));
        }
    }

    out
}

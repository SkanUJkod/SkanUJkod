use crate::cfg::types::{BasicBlock, Statement};
use go_parser::ast::{Stmt, Node};
use go_parser::{AstObjects, FileSet};
use std::collections::HashMap;

/// Get precise line information for a statement
pub fn get_precise_line_info(fset: &FileSet, objs: &AstObjects, stmt: &Stmt) -> (usize, usize) {
    let pos = stmt.pos(objs);
    let end_pos = stmt.end(objs);

    let start_line = fset.position(pos).map(|p| p.line).unwrap_or(0);
    let end_line = fset.position(end_pos).map(|p| p.line).unwrap_or(start_line);

    (start_line, end_line)
}

/// Push a single statement into a basic block
pub fn push_single(blocks: &mut HashMap<usize, BasicBlock>, id: usize, stmt: Stmt) {
    let stmt_text = format!("{:?}", stmt);
    blocks.insert(
        id,
        BasicBlock {
            id,
            stmts: vec![Statement {
                text: stmt_text,
                stmt,
            }],
            succs: Vec::new(),
            start_line: 0, // Will be set properly with fset and objs
            end_line: 0,   // Will be set properly with fset and objs
        },
    );
}

/// Push a single statement into a basic block with line info
pub fn push_single_with_lines(
    fset: &FileSet,
    objs: &AstObjects,
    blocks: &mut HashMap<usize, BasicBlock>,
    id: usize,
    stmt: Stmt,
) {
    let (start_line, end_line) = get_precise_line_info(fset, objs, &stmt);
    let stmt_text = format!("{:?}", stmt);
    blocks.insert(
        id,
        BasicBlock {
            id,
            stmts: vec![Statement {
                text: stmt_text,
                stmt,
            }],
            succs: Vec::new(),
            start_line,
            end_line,
        },
    );
}

/// Macro to connect blocks if the source block has no successors
#[macro_export]
macro_rules! connect_if_empty {
    ($from:expr, $to:expr, $blocks:expr) => {{
        if let Some(b) = $blocks.get_mut(&$from) {
            if b.succs.is_empty() {
                b.succs = vec![$to];
            }
        }
    }};
}

// Re-export the macro for use in other modules
pub use connect_if_empty;

/// Resolve pending goto statements
pub fn resolve_pending_gotos(
    pending_gotos: Vec<(usize, String)>,
    label_map: &HashMap<String, usize>,
    blocks: &mut HashMap<usize, BasicBlock>,
    exit_id: usize,
) {
    for (blk_id, label_name) in pending_gotos {
        if let Some(&target) = label_map.get(&label_name) {
            if let Some(goto_block) = blocks.get_mut(&blk_id) {
                goto_block.succs = vec![target];
            }
        } else {
            eprintln!(
                "Warning: unresolved label '{}' in block {}",
                label_name, blk_id
            );
            // Connect to exit as fallback
            if let Some(goto_block) = blocks.get_mut(&blk_id) {
                goto_block.succs = vec![exit_id];
            }
        }
    }
}

/// Connect the last block to exit if it has no successors
pub fn connect_to_exit_if_needed(
    prev_id: usize,
    exit_id: usize,
    blocks: &mut HashMap<usize, BasicBlock>,
) {
    if prev_id != exit_id {
        if let Some(prev_block) = blocks.get_mut(&prev_id) {
            if prev_block.succs.is_empty() {
                prev_block.succs.push(exit_id);
            }
        }
    }
}

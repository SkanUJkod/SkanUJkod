use crate::cfg::types::{BasicBlock, ControlFlowGraph};
use go_parser::ast::FuncDecl;
use go_parser::{AstObjects, FileSet};
use std::collections::HashMap;

use super::utils::{resolve_pending_gotos, connect_to_exit_if_needed};

impl ControlFlowGraph {
    pub fn build(fset: &FileSet, func: &FuncDecl, objs: &AstObjects) -> Self {
        let mut blocks = HashMap::<usize, BasicBlock>::new();
        let mut next_id = 0;
        let entry = next_id;
        next_id += 1;

        let func_start_line = fset.position(func.pos(objs)).map(|p| p.line).unwrap_or(0);

        blocks.insert(
            entry,
            BasicBlock {
                id: entry,
                stmts: Vec::new(),
                succs: Vec::new(),
                start_line: func_start_line,
                end_line: func_start_line,
            },
        );

        let exit_id = next_id;
        next_id += 1;

        let func_end_line = if let Some(body) = &func.body {
            fset.position(body.end())
                .map(|p| p.line)
                .unwrap_or(func_start_line)
        } else {
            func_start_line
        };

        blocks.insert(
            exit_id,
            BasicBlock {
                id: exit_id,
                stmts: vec![],
                succs: vec![],
                start_line: func_end_line,
                end_line: func_end_line,
            },
        );

        if let Some(body) = &func.body {
            let mut label_map = HashMap::new();
            let (first_id, last_id, pending_gotos) = Self::build_chain(
                objs,
                &mut blocks,
                &mut next_id,
                &body.list,
                exit_id,
                None,
                None,
                &mut label_map,
            );

            if let Some(entry_block) = blocks.get_mut(&entry) {
                entry_block.succs = vec![first_id];
            }

            connect_to_exit_if_needed(last_id, exit_id, &mut blocks);
            resolve_pending_gotos(pending_gotos, &label_map, &mut blocks, exit_id);
        } else {
            if let Some(entry_block) = blocks.get_mut(&entry) {
                entry_block.succs = vec![exit_id];
            }
        }

        ControlFlowGraph {
            blocks,
            entry,
        }
    }
}

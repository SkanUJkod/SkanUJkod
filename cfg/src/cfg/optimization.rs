use crate::cfg::types::BasicBlock;
use go_parser::ast::Stmt;
use std::collections::{HashMap, HashSet};

// Clean up unreachable blocks
pub fn clean_unreachable_blocks(blocks: &mut HashMap<usize, BasicBlock>, entry: usize) {
    let mut visited = HashSet::new();
    let mut stack = vec![entry];
    while let Some(id) = stack.pop() {
        if !visited.insert(id) {
            continue;
        }
        if let Some(block) = blocks.get(&id) {
            for &succ in &block.succs {
                stack.push(succ);
            }
        }
    }
    blocks.retain(|id, _| visited.contains(id));
}

// Optimize empty blocks
pub fn optimize_empty_blocks(
    blocks: &mut HashMap<usize, BasicBlock>,
    entry: usize,
    exit_id: usize,
    label_def_ids: &HashSet<usize>,
) {
    let mut changed = true;
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 10;

    while changed && iterations < MAX_ITERATIONS {
        changed = false;
        iterations += 1;

        let mut collapse_pairs = Vec::new();

        for (bid, blk) in &*blocks {
            if *bid == entry || *bid == exit_id {
                continue;
            }

            let is_trivial = blk.stmts.len() == 1
                && matches!(&blk.stmts[0].stmt, Stmt::Empty(e) if e.implicit)
                && blk.succs.len() == 1;

            let is_label_def = label_def_ids.contains(bid);

            if is_trivial && !is_label_def {
                let succ = blk.succs[0];
                if succ != *bid && blocks.contains_key(&succ) {
                    let incoming_count = blocks.values().filter(|b| b.succs.contains(bid)).count();
                    if incoming_count > 0 {
                        collapse_pairs.push((*bid, succ));
                    }
                }
            }
        }

        if !collapse_pairs.is_empty() {
            changed = true;

            for (rid, succ) in collapse_pairs {
                for block in blocks.values_mut() {
                    for s in block.succs.iter_mut() {
                        if *s == rid {
                            *s = succ;
                        }
                    }

                    block.succs.sort_unstable();
                    block.succs.dedup();
                }

                blocks.remove(&rid);
            }
        }
    }

    if iterations >= MAX_ITERATIONS {
        eprintln!("Warning: Empty block optimization reached iteration limit");
    }
}

// cfg/optimization.rs
use super::basic_block::BasicBlock;
use super::graph::ControlFlowGraph;
use std::collections::HashSet;

/// Optimizer for Control Flow Graphs
pub struct CfgOptimizer;

impl CfgOptimizer {
    /// Main optimization entry point
    pub fn optimize(cfg: &mut ControlFlowGraph) {
        // First pass: remove unreachable blocks
        cfg.remove_unreachable_blocks();

        // Second pass: optimize empty blocks
        Self::optimize_empty_blocks(cfg);

        // Final pass: remove unreachable blocks again after optimization
        cfg.remove_unreachable_blocks();

        // Ensure exit block has no successors
        if let Some(exit_block) = cfg.blocks.get_mut(&1) {
            exit_block.succs.clear();
        }
    }

    /// Optimize empty blocks by collapsing trivial ones
    fn optimize_empty_blocks(cfg: &mut ControlFlowGraph) {
        let mut changed = true;
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10;

        while changed && iterations < MAX_ITERATIONS {
            changed = false;
            iterations += 1;

            let mut collapse_pairs = Vec::new();

            // Find blocks that can be collapsed
            let all_bids: Vec<usize> = cfg.blocks.keys().cloned().collect();
            for bid in &all_bids {
                if let Some(block) = cfg.blocks.get(bid) {
                    // Skip entry and exit blocks
                    if *bid == cfg.entry || *bid == 1 {
                        continue;
                    }

                    // Check if this is a trivial empty block
                    if block.is_trivial_empty() {
                        let succ = block.succs[0];

                        // Don't collapse if it would create a self-loop
                        if succ != *bid && cfg.blocks.contains_key(&succ) {
                            // Count incoming edges
                            let incoming_count = cfg
                                .blocks
                                .values()
                                .filter(|b| b.succs.contains(bid))
                                .count();

                            // Only collapse if there are predecessors
                            if incoming_count > 0 {
                                // Check if this block is a label definition
                                // (In the original code, this is tracked by label_def_ids)
                                // For now, we'll assume labeled blocks have special handling
                                // and shouldn't be collapsed
                                let is_label = block
                                    .stmts
                                    .iter()
                                    .any(|stmt| matches!(stmt, go_parser::ast::Stmt::Labeled(_)));

                                if !is_label {
                                    collapse_pairs.push((*bid, succ));
                                }
                            }
                        }
                    }
                }
            }

            // Apply the collapses
            if !collapse_pairs.is_empty() {
                changed = true;

                for (remove_id, target_id) in collapse_pairs {
                    // Redirect all references to the removed block
                    for block in cfg.blocks.values_mut() {
                        // Update successors
                        for succ in block.succs.iter_mut() {
                            if *succ == remove_id {
                                *succ = target_id;
                            }
                        }

                        // Remove duplicates and sort
                        block.succs.sort_unstable();
                        block.succs.dedup();
                    }

                    // Remove the collapsed block
                    cfg.blocks.remove(&remove_id);
                }
            }
        }

        if iterations >= MAX_ITERATIONS {
            eprintln!("Warning: Empty block optimization reached iteration limit");
        }
    }

    /// Merge consecutive blocks where possible
    pub fn merge_consecutive_blocks(cfg: &mut ControlFlowGraph) {
        let mut changed = true;

        while changed {
            changed = false;
            let mut merge_pairs = Vec::new();

            for (block_id, block) in &cfg.blocks {
                // Can only merge if block has exactly one successor
                if block.succs.len() != 1 {
                    continue;
                }

                let succ_id = block.succs[0];

                // Check if successor has exactly one predecessor
                let pred_count = cfg.in_degree(succ_id);
                if pred_count != 1 {
                    continue;
                }

                // Don't merge with entry or exit blocks
                if succ_id == cfg.entry || succ_id == 1 || *block_id == 1 {
                    continue;
                }

                // Don't create self-loops
                if succ_id == *block_id {
                    continue;
                }

                merge_pairs.push((*block_id, succ_id));
            }

            // Apply merges
            for (block_id, succ_id) in merge_pairs {
                if let Some(succ_block) = cfg.blocks.remove(&succ_id) {
                    if let Some(block) = cfg.blocks.get_mut(&block_id) {
                        // Merge statements
                        block.stmts.extend(succ_block.stmts);

                        // Update successors
                        block.succs = succ_block.succs;

                        // Update line numbers
                        if succ_block.end_line > block.end_line {
                            block.end_line = succ_block.end_line;
                        }

                        changed = true;
                    }
                }
            }
        }
    }

    /// Remove redundant jumps (blocks that only jump to another block)
    pub fn remove_redundant_jumps(cfg: &mut ControlFlowGraph) {
        let mut changed = true;

        while changed {
            changed = false;
            // 1) Collect updates using immutable borrows
            let all_ids: Vec<usize> = cfg.blocks.keys().cloned().collect();
            let mut update_pairs = Vec::new();
            for block_id in &all_ids {
                if let Some(block) = cfg.blocks.get(block_id) {
                    if block.succs.len() == 1 {
                        let target = block.succs[0];
                        if let Some(target_block) = cfg.blocks.get(&target) {
                            if target_block.stmts.len() == 1
                                && target_block.succs.len() == 1
                                && matches!(&target_block.stmts[0],
                                    go_parser::ast::Stmt::Empty(e) if e.implicit)
                            {
                                let final_target = target_block.succs[0];
                                if final_target != *block_id && final_target != target {
                                    update_pairs.push((*block_id, final_target));
                                }
                            }
                        }
                    }
                }
            }
            // 2) Apply updates with mutable borrows
            for (block_id, final_target) in update_pairs {
                if let Some(block) = cfg.blocks.get_mut(&block_id) {
                    block.succs[0] = final_target;
                    changed = true;
                }
            }
        }
    }

    /// Validate that all successor references are valid
    pub fn validate_successors(cfg: &ControlFlowGraph) {
        for (id, block) in &cfg.blocks {
            for &succ in &block.succs {
                if !cfg.blocks.contains_key(&succ) {
                    eprintln!(
                        "Warning: block {} references non-existent successor {}",
                        id, succ
                    );
                }
            }
        }
    }

    /// Dead code elimination - remove blocks that cannot reach exit
    pub fn eliminate_dead_code(cfg: &mut ControlFlowGraph) {
        // Find blocks that can reach exit
        let mut can_reach_exit = HashSet::new();
        let mut changed = true;

        // Exit block can reach itself
        can_reach_exit.insert(1);

        // Iteratively find blocks that can reach exit
        while changed {
            changed = false;

            for (block_id, block) in &cfg.blocks {
                if !can_reach_exit.contains(block_id) {
                    // Check if any successor can reach exit
                    for &succ in &block.succs {
                        if can_reach_exit.contains(&succ) {
                            can_reach_exit.insert(*block_id);
                            changed = true;
                            break;
                        }
                    }
                }
            }
        }

        // Remove blocks that cannot reach exit (except entry)
        let mut dead_blocks = Vec::new();
        for &block_id in cfg.blocks.keys() {
            if block_id != cfg.entry && !can_reach_exit.contains(&block_id) {
                dead_blocks.push(block_id);
            }
        }

        for block_id in dead_blocks {
            cfg.blocks.remove(&block_id);
        }
    }
}

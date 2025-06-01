// cfg/validation.rs
use super::graph::ControlFlowGraph;

/// Validator for Control Flow Graphs
pub struct CfgValidator;

impl CfgValidator {
    /// Validate the entire CFG structure
    pub fn validate(cfg: &ControlFlowGraph) {
        Self::validate_entry_exit(cfg);
        Self::validate_successors(cfg);
        Self::validate_reachability(cfg);
        Self::validate_terminal_blocks(cfg);
    }

    /// Validate that entry and exit blocks exist
    fn validate_entry_exit(cfg: &ControlFlowGraph) {
        if !cfg.blocks.contains_key(&cfg.entry) {
            eprintln!("Warning: CFG entry block {} does not exist", cfg.entry);
        }

        if !cfg.blocks.contains_key(&1) {
            eprintln!("Warning: CFG exit block 1 does not exist");
        }
    }

    /// Validate that all successor references are valid
    fn validate_successors(cfg: &ControlFlowGraph) {
        for (id, block) in &cfg.blocks {
            if block.succs.is_empty() && *id != 1 {
                eprintln!("Warning: block {} has no successors", id);
            }

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

    /// Validate that all blocks are reachable from entry
    fn validate_reachability(cfg: &ControlFlowGraph) {
        let reachable = cfg.reachable_blocks();

        for &block_id in cfg.blocks.keys() {
            if !reachable.contains(&block_id) {
                eprintln!("Warning: block {} is unreachable from entry", block_id);
            }
        }
    }

    /// Validate terminal blocks
    fn validate_terminal_blocks(cfg: &ControlFlowGraph) {
        // Exit block should have no successors
        if let Some(exit_block) = cfg.blocks.get(&1) {
            if !exit_block.succs.is_empty() {
                eprintln!(
                    "Warning: exit block has {} successors",
                    exit_block.succs.len()
                );
            }
        }

        // Check for unexpected terminal blocks
        let terminal_blocks = cfg.terminal_blocks();
        for &block_id in &terminal_blocks {
            if block_id != 1 {
                // Terminal blocks other than exit might indicate issues
                eprintln!(
                    "Warning: block {} is terminal but not the exit block",
                    block_id
                );
            }
        }
    }

    /// Check for self-loops
    pub fn check_self_loops(cfg: &ControlFlowGraph) {
        for (id, block) in &cfg.blocks {
            if block.succs.contains(id) {
                eprintln!("Warning: block {} has a self-loop", id);
            }
        }
    }

    /// Check for duplicate successors
    pub fn check_duplicate_successors(cfg: &ControlFlowGraph) {
        for (id, block) in &cfg.blocks {
            let mut seen = std::collections::HashSet::new();
            for &succ in &block.succs {
                if !seen.insert(succ) {
                    eprintln!("Warning: block {} has duplicate successor {}", id, succ);
                }
            }
        }
    }
}

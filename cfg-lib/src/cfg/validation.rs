use crate::cfg::types::BasicBlock;
use std::collections::HashMap;

// Validate the graph
pub fn validate_graph(blocks: &HashMap<usize, BasicBlock>, exit_id: usize) {
    for (id, block) in blocks {
        if block.succs.is_empty() && *id != exit_id {
            eprintln!("Warning: block {} has no successors", id);
        }

        for &succ in &block.succs {
            if !blocks.contains_key(&succ) {
                eprintln!(
                    "Warning: block {} references non-existent successor {}",
                    id, succ
                );
            }
        }
    }
}

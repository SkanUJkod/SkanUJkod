// cfg/graph.rs
use super::basic_block::BasicBlock;
use super::builder::CfgBuilder;
use go_parser::ast::FuncDecl;
use go_parser::{AstObjects, FileSet};
use std::collections::{HashMap, HashSet};

/// Control Flow Graph representation
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// All basic blocks in the graph, indexed by their ID
    pub blocks: HashMap<usize, BasicBlock>,
    /// Entry block ID
    pub entry: usize,
}

impl ControlFlowGraph {
    /// Create a new empty CFG
    pub fn new(entry: usize) -> Self {
        ControlFlowGraph {
            blocks: HashMap::new(),
            entry,
        }
    }

    /// Build a CFG from a function declaration
    pub fn build(fset: &FileSet, func: &FuncDecl, objs: &AstObjects) -> Self {
        CfgBuilder::new(fset, objs).build_function(func)
    }

    /// Get the exit block ID if it exists
    pub fn exit_block(&self) -> Option<usize> {
        // Exit block is typically block with ID 1 in our convention
        if self.blocks.contains_key(&1) {
            Some(1)
        } else {
            None
        }
    }

    /// Get all reachable blocks from the entry point
    pub fn reachable_blocks(&self) -> HashSet<usize> {
        let mut visited = HashSet::new();
        let mut stack = vec![self.entry];

        while let Some(id) = stack.pop() {
            if !visited.insert(id) {
                continue;
            }

            if let Some(block) = self.blocks.get(&id) {
                for &succ in &block.succs {
                    stack.push(succ);
                }
            }
        }

        visited
    }

    /// Remove unreachable blocks from the graph
    pub fn remove_unreachable_blocks(&mut self) {
        let reachable = self.reachable_blocks();
        self.blocks.retain(|id, _| reachable.contains(id));
    }

    /// Get all blocks that have no successors (terminal blocks)
    pub fn terminal_blocks(&self) -> Vec<usize> {
        self.blocks
            .iter()
            .filter(|(_, block)| block.is_terminal())
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get all blocks that reference a specific block as successor
    pub fn predecessors(&self, block_id: usize) -> Vec<usize> {
        self.blocks
            .iter()
            .filter(|(_, block)| block.succs.contains(&block_id))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get the number of incoming edges for a block
    pub fn in_degree(&self, block_id: usize) -> usize {
        self.blocks
            .values()
            .filter(|block| block.succs.contains(&block_id))
            .count()
    }

    /// Compute dominators for all blocks
    pub fn compute_dominators(&self) -> HashMap<usize, HashSet<usize>> {
        let mut dominators: HashMap<usize, HashSet<usize>> = HashMap::new();

        // Entry block only dominated by itself
        dominators.insert(self.entry, HashSet::from([self.entry]));

        // Initially, all other blocks are dominated by all blocks
        let all_blocks: HashSet<usize> = self.blocks.keys().cloned().collect();
        for &block_id in self.blocks.keys() {
            if block_id != self.entry {
                dominators.insert(block_id, all_blocks.clone());
            }
        }

        // Iteratively refine dominators
        let mut changed = true;
        while changed {
            changed = false;

            for &block_id in self.blocks.keys() {
                if block_id == self.entry {
                    continue;
                }

                let preds = self.predecessors(block_id);
                if preds.is_empty() {
                    continue;
                }

                // Compute intersection of predecessors' dominators
                let mut new_doms = all_blocks.clone();
                for pred in preds {
                    if let Some(pred_doms) = dominators.get(&pred) {
                        new_doms = new_doms.intersection(pred_doms).cloned().collect();
                    }
                }
                new_doms.insert(block_id);

                if let Some(current_doms) = dominators.get(&block_id) {
                    if &new_doms != current_doms {
                        dominators.insert(block_id, new_doms);
                        changed = true;
                    }
                }
            }
        }

        dominators
    }

    /// Find natural loops in the CFG
    pub fn find_loops(&self) -> Vec<Loop> {
        let dominators = self.compute_dominators();
        let mut loops = Vec::new();

        // Find back edges
        for (block_id, block) in &self.blocks {
            for &succ in &block.succs {
                if let Some(succ_doms) = dominators.get(block_id) {
                    if succ_doms.contains(&succ) {
                        // Found a back edge from block_id to succ
                        let loop_blocks = self.find_loop_blocks(succ, *block_id);
                        loops.push(Loop {
                            header: succ,
                            blocks: loop_blocks,
                        });
                    }
                }
            }
        }

        loops
    }

    /// Find all blocks in a natural loop given header and a back edge source
    fn find_loop_blocks(&self, header: usize, back_edge_source: usize) -> HashSet<usize> {
        let mut loop_blocks = HashSet::from([header, back_edge_source]);
        let mut stack = vec![back_edge_source];

        while let Some(block) = stack.pop() {
            let preds = self.predecessors(block);
            for pred in preds {
                if loop_blocks.insert(pred) {
                    stack.push(pred);
                }
            }
        }

        loop_blocks
    }
}

/// Represents a natural loop in the CFG
#[derive(Debug, Clone)]
pub struct Loop {
    /// The header block of the loop
    pub header: usize,
    /// All blocks that are part of the loop
    pub blocks: HashSet<usize>,
}

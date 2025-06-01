// cfg/basic_block.rs
use go_parser::ast::Stmt;

/// Represents a basic block in the control flow graph
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Unique identifier for the block
    pub id: usize,
    /// List of statements in the block
    pub stmts: Vec<Stmt>,
    /// Successor block IDs
    pub succs: Vec<usize>,
    /// Starting line number of the block
    pub start_line: usize,
    /// Ending line number of the block
    pub end_line: usize,
}

impl BasicBlock {
    /// Create a new basic block
    pub fn new(id: usize) -> Self {
        BasicBlock {
            id,
            stmts: Vec::new(),
            succs: Vec::new(),
            start_line: 0,
            end_line: 0,
        }
    }

    /// Create a new basic block with a single statement
    pub fn with_stmt(id: usize, stmt: Stmt, line: usize) -> Self {
        BasicBlock {
            id,
            stmts: vec![stmt],
            succs: Vec::new(),
            start_line: line,
            end_line: line,
        }
    }

    /// Create an empty basic block (implicit empty statement)
    pub fn empty(id: usize) -> Self {
        use go_parser::ast::EmptyStmt;
        use std::rc::Rc;

        BasicBlock {
            id,
            stmts: vec![Stmt::Empty(Rc::new(EmptyStmt {
                semi: 0,
                implicit: true,
            }))],
            succs: Vec::new(),
            start_line: 0,
            end_line: 0,
        }
    }

    /// Check if this block has no successors
    pub fn is_terminal(&self) -> bool {
        self.succs.is_empty()
    }

    /// Check if this block is trivial (single implicit empty statement)
    pub fn is_trivial_empty(&self) -> bool {
        self.stmts.len() == 1
            && matches!(&self.stmts[0], Stmt::Empty(e) if e.implicit)
            && self.succs.len() == 1
    }

    /// Add a successor to this block
    pub fn add_successor(&mut self, succ_id: usize) {
        if !self.succs.contains(&succ_id) {
            self.succs.push(succ_id);
        }
    }

    /// Set successors, replacing any existing ones
    pub fn set_successors(&mut self, succs: Vec<usize>) {
        self.succs = succs;
    }

    /// Connect this block to another if it has no successors
    pub fn connect_if_terminal(&mut self, target: usize) {
        if self.succs.is_empty() {
            self.succs.push(target);
        }
    }
}

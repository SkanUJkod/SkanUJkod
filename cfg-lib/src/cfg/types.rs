use go_parser::ast::Stmt;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,
    pub stmts: Vec<Stmt>,
    pub succs: Vec<usize>,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub blocks: HashMap<usize, BasicBlock>,
    pub entry: usize,
}

#[derive(Debug, Clone)]
pub struct LoopContext {
    pub label_id: usize,
    pub cond_id: usize,
    pub after_id: usize,
}

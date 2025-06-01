// cfg/builder.rs
use super::basic_block::BasicBlock;
use super::graph::ControlFlowGraph;
use super::optimization::CfgOptimizer;
use super::stmt_handlers::StmtHandlers;
use super::validation::CfgValidator;
use go_parser::ast::{BlockStmt, FuncDecl, Stmt, Node};
use std::rc::Rc;
use go_parser::ast::EmptyStmt;
use go_parser::{AstObjects, FileSet};
use std::collections::{HashMap, HashSet};

/// Context for loops during CFG construction
#[derive(Debug, Clone)]
pub struct LoopContext {
    pub label_id: usize,
    pub cond_id: usize,
    pub after_id: usize,
}

/// Context for building the CFG
pub struct BuildContext {
    pub next_id: usize,
    pub exit_id: usize,
    pub label_map: HashMap<String, usize>,
    pub loop_contexts: HashMap<String, LoopContext>,
    pub label_def_ids: HashSet<usize>,
    pub pending_gotos: Vec<(usize, String)>,
    pub current_loop_cond: Option<usize>,
    pub current_loop_after: Option<usize>,
}

impl BuildContext {
    fn new(exit_id: usize) -> Self {
        BuildContext {
            next_id: 2, // 0 is entry, 1 is exit
            exit_id,
            label_map: HashMap::new(),
            loop_contexts: HashMap::new(),
            label_def_ids: HashSet::new(),
            pending_gotos: Vec::new(),
            current_loop_cond: None,
            current_loop_after: None,
        }
    }

    pub fn next_block_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn push_loop_context(
        &mut self,
        cond: usize,
        after: usize,
    ) -> (Option<usize>, Option<usize>) {
        let saved = (self.current_loop_cond, self.current_loop_after);
        self.current_loop_cond = Some(cond);
        self.current_loop_after = Some(after);
        saved
    }

    pub fn pop_loop_context(&mut self, saved: (Option<usize>, Option<usize>)) {
        (self.current_loop_cond, self.current_loop_after) = saved;
    }
}

/// Builder for Control Flow Graphs
pub struct CfgBuilder<'a> {
    fset: &'a FileSet,
    objs: &'a AstObjects,
    handlers: StmtHandlers<'a>,
}

impl<'a> CfgBuilder<'a> {
    pub fn new(fset: &'a FileSet, objs: &'a AstObjects) -> Self {
        CfgBuilder {
            fset,
            objs,
            handlers: StmtHandlers::new(fset, objs),
        }
    }

    /// Build a CFG for a function
    pub fn build_function(&self, func: &FuncDecl) -> ControlFlowGraph {
        let mut cfg = ControlFlowGraph::new(0);
        let mut context = BuildContext::new(1);

        // Create entry block
        cfg.blocks.insert(0, BasicBlock::new(0));

        // Create exit block
        cfg.blocks.insert(1, BasicBlock::new(1));

        // Handle function body if present
        if let Some(body_rc) = &func.body {
            let body: &BlockStmt = body_rc.as_ref();
            let prev_id = self.build_block_statements(&mut cfg, &mut context, 0, &body.list);

            // Connect last block to exit if needed
            self.connect_to_exit_if_needed(&mut cfg, &context, prev_id);
        } else {
            // No body, connect entry to exit
            cfg.blocks.get_mut(&0).unwrap().add_successor(1);
        }

        // Resolve pending gotos
        self.resolve_pending_gotos(&mut cfg, &mut context);

        // Optimize the CFG
        CfgOptimizer::optimize(&mut cfg);

        // Validate the CFG
        CfgValidator::validate(&cfg);

        cfg
    }

    /// Build statements in a block
    fn build_block_statements(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &mut BuildContext,
        mut prev_id: usize,
        stmts: &[Stmt],
    ) -> usize {
        for stmt in stmts {
            prev_id = self.handlers.handle_statement(cfg, context, prev_id, stmt);
        }
        prev_id
    }

    /// Build a chain of statements (used for nested blocks)
    pub fn build_statement_chain(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &mut BuildContext,
        stmts: &[Stmt],
    ) -> (usize, usize) {
        if stmts.is_empty() {
            let id = context.next_block_id();
            cfg.blocks.insert(id, BasicBlock::empty(id));
            return (id, id);
        }

        let entry = context.next_block_id();
        let mut prev_id = entry;
        let mut first = true;

        for stmt in stmts {
            if first {
                first = false;
                // Handle the first statement specially to establish entry
                prev_id = self
                    .handlers
                    .handle_statement_with_id(cfg, context, entry, stmt);
            } else {
                prev_id = self.handlers.handle_statement(cfg, context, prev_id, stmt);
            }
        }

        (entry, prev_id)
    }

    /// Connect block to exit if it has no successors
    fn connect_to_exit_if_needed(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &BuildContext,
        block_id: usize,
    ) {
        if block_id != context.exit_id {
            if let Some(block) = cfg.blocks.get_mut(&block_id) {
                if block.is_terminal() {
                    block.add_successor(context.exit_id);
                }
            }
        }
    }

    /// Resolve all pending goto statements
    fn resolve_pending_gotos(&self, cfg: &mut ControlFlowGraph, context: &mut BuildContext) {
        for (block_id, label_name) in &context.pending_gotos {
            if let Some(&target) = context.label_map.get(label_name) {
                if let Some(goto_block) = cfg.blocks.get_mut(block_id) {
                    goto_block.set_successors(vec![target]);
                }
            } else {
                eprintln!(
                    "Warning: unresolved label '{}' in block {}",
                    label_name, block_id
                );
                // Point to exit as fallback
                if let Some(goto_block) = cfg.blocks.get_mut(block_id) {
                    goto_block.set_successors(vec![context.exit_id]);
                }
            }
        }
    }

    /// Create a basic block with a single statement, given its id
    pub fn create_stmt_block(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &mut BuildContext,
        id: usize,
        stmt: Stmt,
    ) {
        let pos = stmt.pos(self.objs);
        let line = self.fset.position(pos).unwrap().line;
        cfg.blocks.insert(id, BasicBlock::with_stmt(id, stmt, line));
    }

    /// Create a basic block with an empty statement, given its id
    pub fn create_empty_block(
        &self,
        cfg: &mut ControlFlowGraph,
        id: usize,
    ) {
        // Insert an explicit empty statement (so succs and stmts align with the original full-code behavior):
        let empty_stmt = Stmt::Empty(Rc::new(EmptyStmt { semi: 0, implicit: true }));
        let pos = empty_stmt.pos(self.objs);
        let line = self.fset.position(pos).unwrap().line;
        cfg.blocks.insert(id, BasicBlock::with_stmt(id, empty_stmt, line));
    }

    /// Connect two blocks if the source has no successors
    pub fn connect_if_empty(cfg: &mut ControlFlowGraph, from: usize, to: usize) {
        if let Some(block) = cfg.blocks.get_mut(&from) {
            if block.is_terminal() {
                block.add_successor(to);
            }
        }
    }
}

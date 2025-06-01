// cfg/stmt_handlers.rs
use super::basic_block::BasicBlock;
use super::builder::{BuildContext, CfgBuilder, LoopContext};
use super::graph::ControlFlowGraph;
use go_parser::ast::{
    BlockStmt, BranchStmt, CaseClause, EmptyStmt, Expr, ForStmt, IfStmt, LabeledStmt, Node,
    RangeStmt, ReturnStmt, Stmt, SwitchStmt,
};
use go_parser::{AstObjects, FileSet, Token};
use std::rc::Rc;

/// Handlers for different statement types
pub struct StmtHandlers<'a> {
    fset: &'a FileSet,
    objs: &'a AstObjects,
}

impl<'a> StmtHandlers<'a> {
    pub fn new(fset: &'a FileSet, objs: &'a AstObjects) -> Self {
        StmtHandlers { fset, objs }
    }

    /// Main entry point for handling any statement
    pub fn handle_statement(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &mut BuildContext,
        prev_id: usize,
        stmt: &Stmt,
    ) -> usize {
        match stmt {
            Stmt::If(if_stmt) => self.handle_if(cfg, context, prev_id, if_stmt.clone()),
            Stmt::For(for_stmt) => self.handle_for(cfg, context, prev_id, for_stmt.clone()),
            Stmt::Range(range_stmt) => self.handle_range(cfg, context, prev_id, range_stmt.clone()),
            Stmt::Switch(switch_stmt) => self.handle_switch(cfg, context, prev_id, switch_stmt.clone()),
            Stmt::Labeled(_) => self.handle_labeled(cfg, context, prev_id, stmt),
            Stmt::Branch(branch_stmt) => self.handle_branch(cfg, context, prev_id, branch_stmt.clone()),
            Stmt::Return(return_stmt) => self.handle_return(cfg, context, prev_id, return_stmt.clone()),
            _ => self.handle_simple_stmt(cfg, context, prev_id, stmt),
        }
    }

    /// Handle statement with a specific ID (used for chain building)
    pub fn handle_statement_with_id(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &mut BuildContext,
        id: usize,
        stmt: &Stmt,
    ) -> usize {
        // Create the block first
        let pos = stmt.pos(self.objs);
        let line = self.fset.position(pos).unwrap().line;
        cfg.blocks
            .insert(id, BasicBlock::with_stmt(id, stmt.clone(), line));

        // Then handle specific logic based on statement type
        match stmt {
            Stmt::Branch(branch_stmt) => {
                self.setup_branch_successors(cfg, context, id, branch_stmt);
                id
            }
            Stmt::Return(_) => {
                if let Some(block) = cfg.blocks.get_mut(&id) {
                    block.set_successors(vec![context.exit_id]);
                }
                id
            }
            _ => id,
        }
    }

    /// Handle if statement
    fn handle_if(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &mut BuildContext,
        prev_id: usize,
        if_stmt: Rc<IfStmt>,
    ) -> usize {
        // Create condition block
        let cond_id = context.next_block_id();
        self.create_stmt_block(cfg, context, cond_id, Stmt::If(if_stmt.clone()));

        // Connect previous to condition
        CfgBuilder::connect_if_empty(cfg, prev_id, cond_id);

        // Build then branch
        let builder = CfgBuilder::new(self.fset, self.objs);
        let (then_entry, then_exit) =
            builder.build_statement_chain(cfg, context, &if_stmt.body.list);

        // Build else branch if present
        let else_result = if let Some(else_stmt) = &if_stmt.els {
            let list: Vec<Stmt> = match else_stmt {
                Stmt::Block(bs) => bs.list.clone(),
                Stmt::If(_) => vec![(*else_stmt).clone()],
                _ => vec![],
            };
            if !list.is_empty() {
                let (entry, exit) = builder.build_statement_chain(cfg, context, &list);
                Some((entry, exit))
            } else {
                None
            }
        } else {
            None
        };

        // Create after block
        let after_id = context.next_block_id();
        cfg.blocks.insert(after_id, BasicBlock::empty(after_id));

        // Connect condition to branches
        if let Some(cond_block) = cfg.blocks.get_mut(&cond_id) {
            cond_block.add_successor(then_entry);
            cond_block.add_successor(else_result.map(|(entry, _)| entry).unwrap_or(after_id));
        }

        // Connect branches to after block
        CfgBuilder::connect_if_empty(cfg, then_exit, after_id);
        if let Some((_, else_exit)) = else_result {
            CfgBuilder::connect_if_empty(cfg, else_exit, after_id);
        }

        after_id
    }

    /// Handle for loop
    fn handle_for(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &mut BuildContext,
        prev_id: usize,
        for_stmt: Rc<ForStmt>,
    ) -> usize {
        // Handle initialization if present
        let init_id = if let Some(init_stmt) = &for_stmt.init {
            let id = context.next_block_id();
            self.create_stmt_block(cfg, context, id, init_stmt.clone());
            CfgBuilder::connect_if_empty(cfg, prev_id, id);
            id
        } else {
            prev_id
        };

        // Create condition block
        let cond_id = context.next_block_id();
        let cond_stmt = if let Some(cond_expr) = &for_stmt.cond {
            Stmt::Expr(Box::new(cond_expr.clone()))
        } else {
            Stmt::Empty(Rc::new(EmptyStmt {
                semi: 0,
                implicit: true,
            }))
        };
        self.create_stmt_block(cfg, context, cond_id, cond_stmt);

        // Connect init/prev to condition
        if init_id == prev_id {
            CfgBuilder::connect_if_empty(cfg, prev_id, cond_id);
        } else {
            CfgBuilder::connect_if_empty(cfg, init_id, cond_id);
        }

        // Create after block
        let after_id = context.next_block_id();
        cfg.blocks.insert(after_id, BasicBlock::empty(after_id));

        // Setup loop context
        let saved_context = context.push_loop_context(cond_id, after_id);

        // Build loop body
        let builder = CfgBuilder::new(self.fset, self.objs);
        let (body_entry, body_exit) =
            builder.build_statement_chain(cfg, context, &for_stmt.body.list);

        // Handle post statement if present
        if let Some(post_stmt) = &for_stmt.post {
            let post_id = context.next_block_id();
            self.create_stmt_block(cfg, context, post_id, post_stmt.clone());
            CfgBuilder::connect_if_empty(cfg, body_exit, post_id);
            if let Some(post_block) = cfg.blocks.get_mut(&post_id) {
                post_block.set_successors(vec![cond_id]);
            }
        } else {
            CfgBuilder::connect_if_empty(cfg, body_exit, cond_id);
        }

        // Connect condition to body and after
        if let Some(cond_block) = cfg.blocks.get_mut(&cond_id) {
            cond_block.add_successor(body_entry);
            cond_block.add_successor(after_id);
        }

        // Restore loop context
        context.pop_loop_context(saved_context);

        after_id
    }

    /// Handle range statement
    fn handle_range(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &mut BuildContext,
        prev_id: usize,
        range_stmt: Rc<RangeStmt>,
    ) -> usize {
        // Create condition block for range
        let cond_id = context.next_block_id();
        self.create_stmt_block(cfg, context, cond_id, Stmt::Range(range_stmt.clone()));
        CfgBuilder::connect_if_empty(cfg, prev_id, cond_id);

        // Create after block
        let after_id = context.next_block_id();
        cfg.blocks.insert(after_id, BasicBlock::empty(after_id));

        // Setup loop context
        let saved_context = context.push_loop_context(cond_id, after_id);

        // Build loop body
        let builder = CfgBuilder::new(self.fset, self.objs);
        let (body_entry, body_exit) =
            builder.build_statement_chain(cfg, context, &range_stmt.body.list);

        // Connect body back to condition
        CfgBuilder::connect_if_empty(cfg, body_exit, cond_id);

        // Connect condition to body and after
        if let Some(cond_block) = cfg.blocks.get_mut(&cond_id) {
            cond_block.add_successor(body_entry);
            cond_block.add_successor(after_id);
        }

        // Restore loop context
        context.pop_loop_context(saved_context);

        after_id
    }

    /// Handle switch statement
    fn handle_switch(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &mut BuildContext,
        prev_id: usize,
        switch_stmt: Rc<SwitchStmt>,
    ) -> usize {
        // Create switch block
        let switch_id = context.next_block_id();
        self.create_stmt_block(cfg, context, switch_id, Stmt::Switch(switch_stmt.clone()));
        CfgBuilder::connect_if_empty(cfg, prev_id, switch_id);

        let mut case_info = Vec::new();

        // Build each case
        for case_stmt in &switch_stmt.body.list {
            if let Stmt::Case(case_rc) = case_stmt {
                let case: &CaseClause = case_rc.as_ref();
                let builder = CfgBuilder::new(self.fset, self.objs);
                let (entry, exit) = builder.build_statement_chain(cfg, context, &case.body);

                // Check for fallthrough
                let has_fallthrough = case
                    .body
                    .last()
                    .and_then(|stmt| {
                        if let Stmt::Branch(br) = stmt {
                            Some(br.token == Token::FALLTHROUGH)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(false);

                case_info.push((entry, exit, has_fallthrough));
            }
        }

        // Create after block
        let after_id = context.next_block_id();
        cfg.blocks.insert(after_id, BasicBlock::empty(after_id));

        // Connect switch to cases
        if let Some(switch_block) = cfg.blocks.get_mut(&switch_id) {
            for (entry, _, _) in &case_info {
                switch_block.add_successor(*entry);
            }
            if switch_block.succs.is_empty() {
                switch_block.add_successor(after_id);
            }
        }

        // Handle fallthrough and connect cases to after
        for i in 0..case_info.len() {
            let (_, exit, has_fallthrough) = case_info[i];
            if let Some(exit_block) = cfg.blocks.get_mut(&exit) {
                if exit_block.is_terminal() {
                    if has_fallthrough && i + 1 < case_info.len() {
                        let next_entry = case_info[i + 1].0;
                        exit_block.set_successors(vec![next_entry]);
                    } else {
                        exit_block.set_successors(vec![after_id]);
                    }
                }
            }
        }

        after_id
    }

    /// Handle labeled statement
    fn handle_labeled(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &mut BuildContext,
        prev_id: usize,
        stmt: &Stmt,
    ) -> usize {
        if let Stmt::Labeled(lbl_key) = stmt {
            let label_id = context.next_block_id();
            self.create_stmt_block(cfg, context, label_id, stmt.clone());
            context.label_def_ids.insert(label_id);
            CfgBuilder::connect_if_empty(cfg, prev_id, label_id);

            let lbl = &self.objs.l_stmts[*lbl_key];
            let name = self.objs.idents[lbl.label].name.clone();
            context.label_map.insert(name.clone(), label_id);

            match &lbl.stmt {
                Stmt::For(for_stmt) => {
                    let init_id = label_id;
                    let cond_id = context.next_block_id() + 1;
                    let after_id = cond_id + 1;

                    context.loop_contexts.insert(
                        name,
                        LoopContext {
                            label_id,
                            cond_id,
                            after_id,
                        },
                    );

                    self.handle_for(cfg, context, label_id, for_stmt.clone())
                }
                Stmt::Range(range_stmt) => {
                    self.handle_range(cfg, context, label_id, range_stmt.clone())
                }
                _ => label_id,
            }
        } else {
            prev_id
        }
    }

    /// Handle branch statements (break, continue, goto, fallthrough)
    fn handle_branch(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &mut BuildContext,
        prev_id: usize,
        branch_stmt: Rc<BranchStmt>,
    ) -> usize {
        let id = context.next_block_id();
        self.create_stmt_block(cfg, context, id, Stmt::Branch(branch_stmt.clone()));
        CfgBuilder::connect_if_empty(cfg, prev_id, id);

        self.setup_branch_successors(cfg, context, id, &branch_stmt);
        id
    }

    /// Setup successors for a branch statement
    fn setup_branch_successors(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &mut BuildContext,
        block_id: usize,
        branch_stmt: &BranchStmt,
    ) {
        if let Some(block) = cfg.blocks.get_mut(&block_id) {
            match branch_stmt.token {
                Token::GOTO => {
                    if let Some(label_ident) = branch_stmt.label {
                        let name = self.objs.idents[label_ident].name.clone();
                        if let Some(&target) = context.label_map.get(&name) {
                            block.set_successors(vec![target]);
                        } else {
                            context.pending_gotos.push((block_id, name));
                        }
                    }
                }
                Token::BREAK => {
                    if let Some(label_ident) = branch_stmt.label {
                        let name = self.objs.idents[label_ident].name.clone();
                        if let Some(loop_ctx) = context.loop_contexts.get(&name) {
                            block.set_successors(vec![loop_ctx.after_id]);
                        } else {
                            context.pending_gotos.push((block_id, name));
                        }
                    } else if let Some(after) = context.current_loop_after {
                        block.set_successors(vec![after]);
                    } else {
                        block.set_successors(vec![context.exit_id]);
                    }
                }
                Token::CONTINUE => {
                    if let Some(label_ident) = branch_stmt.label {
                        let name = self.objs.idents[label_ident].name.clone();
                        if let Some(loop_ctx) = context.loop_contexts.get(&name) {
                            block.set_successors(vec![loop_ctx.cond_id]);
                        } else {
                            context.pending_gotos.push((block_id, name));
                        }
                    } else if let Some(cond) = context.current_loop_cond {
                        block.set_successors(vec![cond]);
                    } else {
                        block.set_successors(vec![context.exit_id]);
                    }
                }
                Token::FALLTHROUGH => {
                    // Fallthrough is handled in switch statement processing
                }
                _ => {}
            }
        }
    }

    /// Handle return statement
    fn handle_return(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &mut BuildContext,
        prev_id: usize,
        return_stmt: Rc<ReturnStmt>,
    ) -> usize {
        let id = context.next_block_id();
        self.create_stmt_block(cfg, context, id, Stmt::Return(return_stmt.clone()));
        CfgBuilder::connect_if_empty(cfg, prev_id, id);

        if let Some(block) = cfg.blocks.get_mut(&id) {
            block.set_successors(vec![context.exit_id]);
        }

        id
    }

    /// Handle simple statements
    fn handle_simple_stmt(
        &self,
        cfg: &mut ControlFlowGraph,
        context: &mut BuildContext,
        prev_id: usize,
        stmt: &Stmt,
    ) -> usize {
        let id = context.next_block_id();
        self.create_stmt_block(cfg, context, id, stmt.clone());
        CfgBuilder::connect_if_empty(cfg, prev_id, id);
        id
    }

    /// Create a statement block with proper line numbers
    fn create_stmt_block(
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
}

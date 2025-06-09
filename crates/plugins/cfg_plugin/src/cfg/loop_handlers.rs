use crate::cfg::types::{BasicBlock, ControlFlowGraph, LoopContext, Statement};
use go_parser::ast::{EmptyStmt, Stmt};
use go_parser::AstObjects;
use std::collections::HashMap;
use std::rc::Rc;

use super::utils::{push_single, connect_if_empty};

impl ControlFlowGraph {
    /// Handle for loop statements
    #[allow(dead_code)]
    pub(crate) fn handle_for_statement(
        stmt: &Stmt,
        objs: &AstObjects,
        blocks: &mut HashMap<usize, BasicBlock>,
        next_id: &mut usize,
        prev_id: usize,
        exit_id: usize,
        _current_loop_cond: Option<usize>,
        _current_loop_after: Option<usize>,
        label_map: &mut HashMap<String, usize>,
        pending_gotos: &mut Vec<(usize, String)>,
    ) -> usize {
        if let Stmt::For(forst) = stmt {
            let init_id = if let Some(init_stmt) = &forst.init {
                let id = *next_id;
                *next_id += 1;
                push_single(blocks, id, init_stmt.clone());
                connect_if_empty!(prev_id, id, blocks);
                id
            } else {
                prev_id
            };

            let cond_id = *next_id;
            *next_id += 1;
            let cond_stmt = if let Some(cond_expr) = &forst.cond {
                Stmt::Expr(Box::new(cond_expr.clone()))
            } else {
                Stmt::Empty(Rc::new(EmptyStmt {
                    semi: 0,
                    implicit: true,
                }))
            };
            push_single(blocks, cond_id, cond_stmt);

            if init_id == prev_id {
                connect_if_empty!(prev_id, cond_id, blocks);
            } else {
                connect_if_empty!(init_id, cond_id, blocks);
            }

            let after_id = *next_id;
            *next_id += 1;
            blocks.insert(
                after_id,
                BasicBlock {
                    id: after_id,
                    stmts: vec![Statement {
                        text: "<empty>".to_string(),
                        stmt: Stmt::Empty(Rc::new(EmptyStmt {
                            semi: 0,
                            implicit: true,
                        })),
                    }],
                    succs: vec![],
                    start_line: 0,
                    end_line: 0,
                },
            );

            let mut loop_label_map = label_map.clone();
            let (body_entry, body_exit, mut body_gotos) = Self::build_chain(
                objs,
                blocks,
                next_id,
                &forst.body.list,
                exit_id,
                Some(cond_id),
                Some(after_id),
                &mut loop_label_map,
            );
            for (k, v) in loop_label_map {
                label_map.insert(k, v);
            }
            pending_gotos.append(&mut body_gotos);

            if let Some(post_stmt) = &forst.post {
                let post_id = *next_id;
                *next_id += 1;
                push_single(blocks, post_id, post_stmt.clone());

                if let Some(body_exit_block) = blocks.get_mut(&body_exit) {
                    if body_exit_block.succs.is_empty() {
                        body_exit_block.succs = vec![post_id];
                    }
                }
                if let Some(post_block) = blocks.get_mut(&post_id) {
                    post_block.succs = vec![cond_id];
                }
            } else {
                if let Some(body_exit_block) = blocks.get_mut(&body_exit) {
                    if body_exit_block.succs.is_empty() {
                        body_exit_block.succs = vec![cond_id];
                    }
                }
            }

            if let Some(cond_block) = blocks.get_mut(&cond_id) {
                cond_block.succs.push(body_entry);
                cond_block.succs.push(after_id);
            }

            after_id
        } else {
            prev_id
        }
    }

    /// Handle range loop statements
    #[allow(dead_code)]
    pub(crate) fn handle_range_statement(
        stmt: &Stmt,
        objs: &AstObjects,
        blocks: &mut HashMap<usize, BasicBlock>,
        next_id: &mut usize,
        prev_id: usize,
        exit_id: usize,
        label_map: &mut HashMap<String, usize>,
        pending_gotos: &mut Vec<(usize, String)>,
    ) -> usize {
        if let Stmt::Range(rng) = stmt {
            let cond_id = *next_id;
            *next_id += 1;
            push_single(blocks, cond_id, stmt.clone());

            let after_id = *next_id;
            *next_id += 1;
            blocks.insert(
                after_id,
                BasicBlock {
                    id: after_id,
                    stmts: vec![Statement {
                        text: "<empty>".to_string(),
                        stmt: Stmt::Empty(Rc::new(EmptyStmt {
                            semi: 0,
                            implicit: true,
                        })),
                    }],
                    succs: vec![],
                    start_line: 0,
                    end_line: 0,
                },
            );

            let mut range_label_map = label_map.clone();
            let (body_entry, body_exit, mut range_gotos) = Self::build_chain(
                objs,
                blocks,
                next_id,
                &rng.body.list,
                exit_id,
                Some(cond_id),
                Some(after_id),
                &mut range_label_map,
            );
            for (k, v) in range_label_map {
                label_map.insert(k, v);
            }
            pending_gotos.append(&mut range_gotos);

            if let Some(body_exit_block) = blocks.get_mut(&body_exit) {
                if body_exit_block.succs.is_empty() {
                    body_exit_block.succs = vec![cond_id];
                }
            }

            connect_if_empty!(prev_id, cond_id, blocks);

            if let Some(cond_block) = blocks.get_mut(&cond_id) {
                cond_block.succs.push(body_entry);
                cond_block.succs.push(after_id);
            }

            after_id
        } else {
            prev_id
        }
    }

    /// Handle labeled statements (including labeled loops)
    #[allow(dead_code)]
    pub(crate) fn handle_labeled_statement(
        stmt: &Stmt,
        objs: &AstObjects,
        blocks: &mut HashMap<usize, BasicBlock>,
        next_id: &mut usize,
        prev_id: usize,
        exit_id: usize,
        current_loop_cond: Option<usize>,
        current_loop_after: Option<usize>,
        label_map: &mut HashMap<String, usize>,
        loop_contexts: &mut HashMap<String, LoopContext>,
        label_def_ids: &mut std::collections::HashSet<usize>,
        pending_gotos: &mut Vec<(usize, String)>,
    ) -> usize {
        if let Stmt::Labeled(lbl_key) = stmt {
            let id = *next_id;
            *next_id += 1;
            push_single(blocks, id, stmt.clone());
            label_def_ids.insert(id);
            connect_if_empty!(prev_id, id, blocks);

            let lbl = &objs.l_stmts[*lbl_key];
            let name = objs.idents[lbl.label].name.clone();
            label_map.insert(name.clone(), id);

            let inner_stmt = lbl.stmt.clone();
            match inner_stmt {
                Stmt::For(forst) => {
                    let _prev_saved = prev_id;
                    let current_prev_id = id;

                    let init_id = if let Some(init_stmt) = &forst.init {
                        let fid = *next_id;
                        *next_id += 1;
                        push_single(blocks, fid, init_stmt.clone());
                        connect_if_empty!(current_prev_id, fid, blocks);
                        fid
                    } else {
                        current_prev_id
                    };

                    let cond_id = *next_id;
                    *next_id += 1;
                    let cond_stmt = if let Some(cond_expr) = &forst.cond {
                        Stmt::Expr(Box::new(cond_expr.clone()))
                    } else {
                        Stmt::Empty(Rc::new(EmptyStmt {
                            semi: 0,
                            implicit: true,
                        }))
                    };
                    push_single(blocks, cond_id, cond_stmt);
                    if init_id == current_prev_id {
                        connect_if_empty!(current_prev_id, cond_id, blocks);
                    } else {
                        connect_if_empty!(init_id, cond_id, blocks);
                    }

                    let after_id = *next_id;
                    *next_id += 1;
                    blocks.insert(
                        after_id,
                        BasicBlock {
                            id: after_id,
                            stmts: vec![Statement {
                                text: "<empty>".to_string(),
                                stmt: Stmt::Empty(Rc::new(EmptyStmt {
                                    semi: 0,
                                    implicit: true,
                                })),
                            }],
                            succs: vec![],
                            start_line: 0,
                            end_line: 0,
                        },
                    );

                    loop_contexts.insert(
                        name.clone(),
                        LoopContext {
                            label_id: id,
                            cond_id,
                            after_id,
                        },
                    );

                    let mut loop_label_map = label_map.clone();
                    let (body_entry, body_exit, mut body_gotos) = Self::build_chain(
                        objs,
                        blocks,
                        next_id,
                        &forst.body.list,
                        exit_id,
                        Some(cond_id),
                        Some(after_id),
                        &mut loop_label_map,
                    );

                    for (k, v) in loop_label_map {
                        label_map.insert(k, v);
                    }
                    pending_gotos.append(&mut body_gotos);

                    if let Some(post_stmt) = &forst.post {
                        let post_id = *next_id;
                        *next_id += 1;
                        push_single(blocks, post_id, post_stmt.clone());

                        if let Some(body_exit_block) = blocks.get_mut(&body_exit) {
                            if body_exit_block.succs.is_empty() {
                                body_exit_block.succs = vec![post_id];
                            }
                        }

                        if let Some(post_block) = blocks.get_mut(&post_id) {
                            post_block.succs = vec![cond_id];
                        }
                    } else {
                        if let Some(body_exit_block) = blocks.get_mut(&body_exit) {
                            if body_exit_block.succs.is_empty() {
                                body_exit_block.succs = vec![cond_id];
                            }
                        }
                    }

                    if let Some(cond_block) = blocks.get_mut(&cond_id) {
                        cond_block.succs.push(body_entry);
                        cond_block.succs.push(after_id);
                    }

                    after_id
                }
                Stmt::Range(_) => {
                    let inner_vec = vec![inner_stmt.clone()];
                    let (entry, exit, mut rng_gotos) = Self::build_chain(
                        objs,
                        blocks,
                        next_id,
                        &inner_vec,
                        exit_id,
                        current_loop_cond,
                        current_loop_after,
                        label_map,
                    );
                    pending_gotos.append(&mut rng_gotos);

                    if let Some(lb) = blocks.get_mut(&id) {
                        if lb.succs.is_empty() {
                            lb.succs = vec![entry];
                        }
                    }

                    exit
                }
                _ => {
                    let inner_vec = vec![inner_stmt.clone()];
                    let (entry, exit, mut other_gotos) = Self::build_chain(
                        objs,
                        blocks,
                        next_id,
                        &inner_vec,
                        exit_id,
                        current_loop_cond,
                        current_loop_after,
                        label_map,
                    );
                    pending_gotos.append(&mut other_gotos);

                    if let Some(lb) = blocks.get_mut(&id) {
                        if lb.succs.is_empty() {
                            lb.succs = vec![entry];
                        }
                    }

                    exit
                }
            }
        } else {
            prev_id
        }
    }
}

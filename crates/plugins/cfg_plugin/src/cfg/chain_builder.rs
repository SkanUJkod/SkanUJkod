use crate::cfg::types::{BasicBlock, ControlFlowGraph, Statement};
use go_parser::ast::{EmptyStmt, Stmt};
use go_parser::AstObjects;
use std::collections::HashMap;
use std::rc::Rc;

use super::utils::connect_if_empty;

impl ControlFlowGraph {
    /// Build a chain of statements recursively
    pub(crate) fn build_chain(
        objs: &AstObjects,
        blocks: &mut HashMap<usize, BasicBlock>,
        next_id: &mut usize,
        stmts: &[Stmt],
        exit_id: usize,
        loop_cond: Option<usize>,
        loop_after: Option<usize>,
        label_map: &mut HashMap<String, usize>,
    ) -> (usize, usize, Vec<(usize, String)>) {
        if stmts.is_empty() {
            // For empty statement lists, return the exit_id directly
            // This avoids creating unnecessary intermediate blocks
            return (exit_id, exit_id, Vec::new());
        }

        let mut prev: Option<usize> = None;
        let entry = *next_id;
        let mut last_id = entry;
        let mut pending_gotos: Vec<(usize, String)> = Vec::new();

        for stmt in stmts {
            match stmt {
                Stmt::If(_) => {
                    let prev_id = prev.unwrap_or_else(|| {
                        let id = *next_id;
                        *next_id += 1;
                        blocks.insert(
                            id,
                            BasicBlock {
                                id,
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
                        id
                    });

                    let after_id = Self::handle_if_statement_in_chain(
                        stmt,
                        objs,
                        blocks,
                        next_id,
                        prev_id,
                        exit_id,
                        loop_cond,
                        loop_after,
                        label_map,
                        &mut pending_gotos,
                    );

                    prev = Some(after_id);
                    last_id = after_id;
                }
                Stmt::For(_) => {
                    let prev_id = Self::get_or_create_prev_block(prev, blocks, next_id);

                    let after_id = Self::handle_for_statement_in_chain(
                        stmt,
                        objs,
                        blocks,
                        next_id,
                        prev_id,
                        exit_id,
                        loop_cond,
                        loop_after,
                        label_map,
                        &mut pending_gotos,
                    );

                    prev = Some(after_id);
                    last_id = after_id;
                }
                Stmt::Range(_) => {
                    let prev_id = Self::get_or_create_prev_block(prev, blocks, next_id);

                    let after_id = Self::handle_range_statement_in_chain(
                        stmt,
                        objs,
                        blocks,
                        next_id,
                        prev_id,
                        exit_id,
                        label_map,
                        &mut pending_gotos,
                    );

                    prev = Some(after_id);
                    last_id = after_id;
                }
                Stmt::Labeled(_) => {
                    let id = *next_id;
                    *next_id += 1;
                    blocks.insert(
                        id,
                        BasicBlock {
                            id,
                            stmts: vec![Statement {
                                text: format!("{:?}", stmt),
                                stmt: stmt.clone(),
                            }],
                            succs: vec![],
                            start_line: 0,
                            end_line: 0,
                        },
                    );

                    if let Some(p) = prev {
                        connect_if_empty!(p, id, blocks);
                    }

                    // Handle the labeled statement logic here
                    self::handle_labeled_statement_in_chain(
                        stmt,
                        objs,
                        blocks,
                        next_id,
                        id,
                        exit_id,
                        loop_cond,
                        loop_after,
                        label_map,
                        &mut pending_gotos,
                    );

                    prev = Some(id);
                    last_id = id;
                }
                _ => {
                    let id = *next_id;
                    *next_id += 1;
                    last_id = id;

                    let mut succs = Vec::new();
                    match stmt {
                        Stmt::Return(_) => {
                            succs.push(exit_id);
                        }
                        Stmt::Branch(_) => {
                            // Branch logic will be handled in the statement creation
                            self::handle_branch_in_chain(
                                stmt,
                                objs,
                                &mut succs,
                                exit_id,
                                loop_cond,
                                loop_after,
                                label_map,
                                &mut pending_gotos,
                                id,
                            );
                        }
                        _ => {}
                    };

                    blocks.insert(
                        id,
                        BasicBlock {
                            id,
                            stmts: vec![Statement {
                                text: format!("{:?}", stmt),
                                stmt: stmt.clone(),
                            }],
                            succs,
                            start_line: 0,
                            end_line: 0,
                        },
                    );

                    if let Some(p) = prev {
                        if let Some(prev_block) = blocks.get(&p) {
                            if prev_block.succs.is_empty() {
                                if let Some(prev_block) = blocks.get_mut(&p) {
                                    prev_block.succs = vec![id];
                                }
                            }
                        }
                    }
                    prev = Some(id);
                }
            }
        }

        (entry, last_id, pending_gotos)
    }

    fn get_or_create_prev_block(
        prev: Option<usize>,
        blocks: &mut HashMap<usize, BasicBlock>,
        next_id: &mut usize,
    ) -> usize {
        if let Some(p) = prev {
            p
        } else {
            let id = *next_id;
            *next_id += 1;
            blocks.insert(
                id,
                BasicBlock {
                    id,
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
            id
        }
    }

    fn handle_if_statement_in_chain(
        stmt: &Stmt,
        objs: &AstObjects,
        blocks: &mut HashMap<usize, BasicBlock>,
        next_id: &mut usize,
        prev_id: usize,
        exit_id: usize,
        loop_cond: Option<usize>,
        loop_after: Option<usize>,
        label_map: &mut HashMap<String, usize>,
        pending_gotos: &mut Vec<(usize, String)>,
    ) -> usize {
        // Implementation similar to the main if handler but adapted for build_chain
        if let Stmt::If(ifst) = stmt {
            let cond_id = *next_id;
            *next_id += 1;
            blocks.insert(
                cond_id,
                BasicBlock {
                    id: cond_id,
                    stmts: vec![Statement {
                        text: format!("{:?}", stmt),
                        stmt: stmt.clone(),
                    }],
                    succs: vec![],
                    start_line: 0,
                    end_line: 0,
                },
            );

            if let Some(prev_block) = blocks.get(&prev_id) {
                if prev_block.succs.is_empty() {
                    if let Some(pb) = blocks.get_mut(&prev_id) {
                        pb.succs = vec![cond_id];
                    }
                }
            }

            let mut then_label_map = label_map.clone();
            let (then_entry, then_exit, mut then_gotos) = Self::build_chain(
                objs,
                blocks,
                next_id,
                &ifst.body.list,
                exit_id,
                loop_cond,
                loop_after,
                &mut then_label_map,
            );
            for (k, v) in then_label_map {
                label_map.insert(k, v);
            }
            pending_gotos.append(&mut then_gotos);

            let else_result = if let Some(els_stmt) = &ifst.els {
                let list: Vec<Stmt> = match els_stmt {
                    Stmt::Block(bs) => bs.list.clone(),
                    Stmt::If(_) => vec![(*els_stmt).clone()],
                    _ => vec![],
                };
                if list.is_empty() {
                    None
                } else {
                    let mut else_label_map = label_map.clone();
                    let (entry, exit, mut else_gotos) = Self::build_chain(
                        objs,
                        blocks,
                        next_id,
                        &list,
                        exit_id,
                        loop_cond,
                        loop_after,
                        &mut else_label_map,
                    );
                    for (k, v) in else_label_map {
                        label_map.insert(k, v);
                    }
                    pending_gotos.append(&mut else_gotos);
                    Some((entry, exit))
                }
            } else {
                None
            };

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

            if let Some(cond_block) = blocks.get_mut(&cond_id) {
                cond_block.succs.push(then_entry);
                cond_block
                    .succs
                    .push(else_result.map(|(entry, _)| entry).unwrap_or(after_id));
            }

            if let Some(then_exit_block) = blocks.get_mut(&then_exit) {
                if then_exit_block.succs.is_empty() {
                    then_exit_block.succs = vec![after_id];
                }
            }

            if let Some((_, else_exit)) = else_result {
                if let Some(else_exit_block) = blocks.get_mut(&else_exit) {
                    if else_exit_block.succs.is_empty() {
                        else_exit_block.succs = vec![after_id];
                    }
                }
            }

            after_id
        } else {
            prev_id
        }
    }

    fn handle_for_statement_in_chain(
        stmt: &Stmt,
        objs: &AstObjects,
        blocks: &mut HashMap<usize, BasicBlock>,
        next_id: &mut usize,
        prev_id: usize,
        exit_id: usize,
        _loop_cond: Option<usize>,
        _loop_after: Option<usize>,
        label_map: &mut HashMap<String, usize>,
        pending_gotos: &mut Vec<(usize, String)>,
    ) -> usize {
        // Similar to handle_for_statement but adapted for build_chain context
        if let Stmt::For(forst) = stmt {
            let init_id = if let Some(init_stmt) = &forst.init {
                let id = *next_id;
                *next_id += 1;
                blocks.insert(
                    id,
                    BasicBlock {
                        id,
                        stmts: vec![Statement {
                            text: format!("{:?}", init_stmt),
                            stmt: init_stmt.clone(),
                        }],
                        succs: vec![],
                        start_line: 0,
                        end_line: 0,
                    },
                );
                if let Some(prev_block) = blocks.get(&prev_id) {
                    if prev_block.succs.is_empty() {
                        if let Some(pb) = blocks.get_mut(&prev_id) {
                            pb.succs = vec![id];
                        }
                    }
                }
                id
            } else if let Some(p) = Some(prev_id) {
                p
            } else {
                let id = *next_id;
                *next_id += 1;
                blocks.insert(
                    id,
                    BasicBlock {
                        id,
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
                id
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
            blocks.insert(
                cond_id,
                BasicBlock {
                    id: cond_id,
                    stmts: vec![Statement {
                        text: format!("{:?}", cond_stmt),
                        stmt: cond_stmt,
                    }],
                    succs: vec![],
                    start_line: 0,
                    end_line: 0,
                },
            );
            if let Some(init_block) = blocks.get_mut(&init_id) {
                if init_block.succs.is_empty() {
                    init_block.succs = vec![cond_id];
                }
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

            let mut local_map = label_map.clone();
            let (body_entry, body_exit, mut loop_gotos) = Self::build_chain(
                objs,
                blocks,
                next_id,
                &forst.body.list,
                exit_id,
                Some(cond_id),
                Some(after_id),
                &mut local_map,
            );
            for (k, v) in local_map {
                label_map.insert(k, v);
            }
            pending_gotos.append(&mut loop_gotos);

            if let Some(post_stmt) = &forst.post {
                let post_id = *next_id;
                *next_id += 1;
                blocks.insert(
                    post_id,
                    BasicBlock {
                        id: post_id,
                        stmts: vec![Statement {
                            text: format!("{:?}", post_stmt),
                            stmt: post_stmt.clone(),
                        }],
                        succs: vec![cond_id],
                        start_line: 0,
                        end_line: 0,
                    },
                );
                if let Some(body_exit_block) = blocks.get_mut(&body_exit) {
                    if body_exit_block.succs.is_empty() {
                        body_exit_block.succs = vec![post_id];
                    }
                }
            } else if let Some(body_exit_block) = blocks.get_mut(&body_exit) {
                if body_exit_block.succs.is_empty() {
                    body_exit_block.succs = vec![cond_id];
                }
            }

            if let Some(cond_blk) = blocks.get_mut(&cond_id) {
                cond_blk.succs.push(body_entry);
                cond_blk.succs.push(after_id);
            }

            after_id
        } else {
            prev_id
        }
    }

    fn handle_range_statement_in_chain(
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
            let nested_cond_id = *next_id;
            *next_id += 1;
            blocks.insert(
                nested_cond_id,
                BasicBlock {
                    id: nested_cond_id,
                    stmts: vec![Statement {
                        text: format!("{:?}", stmt),
                        stmt: stmt.clone(),
                    }],
                    succs: vec![],
                    start_line: 0,
                    end_line: 0,
                },
            );

            if let Some(prev_block) = blocks.get(&prev_id) {
                if prev_block.succs.is_empty() {
                    if let Some(prev_block) = blocks.get_mut(&prev_id) {
                        prev_block.succs = vec![nested_cond_id];
                    }
                }
            }

            let nested_after_id = *next_id;
            *next_id += 1;
            blocks.insert(
                nested_after_id,
                BasicBlock {
                    id: nested_after_id,
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

            let mut nested_label_map = label_map.clone();
            let (body_entry, body_exit, mut nested_gotos) = Self::build_chain(
                objs,
                blocks,
                next_id,
                &rng.body.list,
                exit_id,
                Some(nested_cond_id),
                Some(nested_after_id),
                &mut nested_label_map,
            );
            for (k, v) in nested_label_map {
                label_map.insert(k, v);
            }
            pending_gotos.append(&mut nested_gotos);

            if let Some(body_exit_block) = blocks.get_mut(&body_exit) {
                if body_exit_block.succs.is_empty() {
                    body_exit_block.succs = vec![nested_cond_id];
                }
            }

            if let Some(cond_block) = blocks.get_mut(&nested_cond_id) {
                cond_block.succs.push(body_entry);
                cond_block.succs.push(nested_after_id);
            }

            nested_after_id
        } else {
            prev_id
        }
    }
}

fn handle_labeled_statement_in_chain(
    stmt: &Stmt,
    objs: &AstObjects,
    blocks: &mut HashMap<usize, BasicBlock>,
    next_id: &mut usize,
    id: usize,
    exit_id: usize,
    loop_cond: Option<usize>,
    loop_after: Option<usize>,
    label_map: &mut HashMap<String, usize>,
    pending_gotos: &mut Vec<(usize, String)>,
) {
    if let Stmt::Labeled(lbl_key) = stmt {
        let lbl = &objs.l_stmts[*lbl_key];
        let name = objs.idents[lbl.label].name.clone();
        label_map.insert(name.clone(), id);

        let inner_stmt = lbl.stmt.clone();
        match inner_stmt {
            Stmt::For(_) | Stmt::Range(_) => {
                let tmp_vec = vec![inner_stmt];
                let (entry, _exit, mut inner_gotos) = ControlFlowGraph::build_chain(
                    objs, blocks, next_id, &tmp_vec, exit_id, loop_cond, loop_after, label_map,
                );
                pending_gotos.append(&mut inner_gotos);
                if let Some(lb) = blocks.get_mut(&id) {
                    if lb.succs.is_empty() {
                        lb.succs = vec![entry];
                    }
                }
            }
            _ => {
                let tmp_vec = vec![inner_stmt];
                let (entry, _exit, mut inner_gotos) = ControlFlowGraph::build_chain(
                    objs, blocks, next_id, &tmp_vec, exit_id, loop_cond, loop_after, label_map,
                );
                pending_gotos.append(&mut inner_gotos);
                if let Some(lb) = blocks.get_mut(&id) {
                    if lb.succs.is_empty() {
                        lb.succs = vec![entry];
                    }
                }
            }
        }
    }
}

fn handle_branch_in_chain(
    stmt: &Stmt,
    objs: &AstObjects,
    succs: &mut Vec<usize>,
    exit_id: usize,
    loop_cond: Option<usize>,
    loop_after: Option<usize>,
    label_map: &HashMap<String, usize>,
    pending_gotos: &mut Vec<(usize, String)>,
    id: usize,
) {
    use go_parser::Token;
    
    if let Stmt::Branch(br_rc) = stmt {
        use go_parser::ast::BranchStmt;
        let bs: &BranchStmt = br_rc.as_ref();
        match bs.token {
            Token::BREAK => {
                if let Some(label_ident) = bs.label {
                    let name = objs.idents[label_ident].name.clone();
                    if let Some(&target) = label_map.get(&name) {
                        succs.push(target);
                    } else {
                        pending_gotos.push((id, name));
                    }
                } else if let Some(after) = loop_after {
                    succs.push(after);
                } else {
                    succs.push(exit_id);
                }
            }
            Token::CONTINUE => {
                if let Some(label_ident) = bs.label {
                    let name = objs.idents[label_ident].name.clone();
                    if let Some(&target) = label_map.get(&name) {
                        succs.push(target);
                    } else {
                        pending_gotos.push((id, name));
                    }
                } else if let Some(cond) = loop_cond {
                    succs.push(cond);
                } else {
                    succs.push(exit_id);
                }
            }
            Token::GOTO => {
                if let Some(label_ident) = bs.label {
                    let name = objs.idents[label_ident].name.clone();
                    if let Some(&target) = label_map.get(&name) {
                        succs.push(target);
                    } else {
                        pending_gotos.push((id, name));
                    }
                }
            }
            Token::FALLTHROUGH => {}
            _ => {}
        }
    }
}

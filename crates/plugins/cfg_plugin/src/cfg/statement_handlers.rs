use crate::cfg::types::{BasicBlock, ControlFlowGraph, LoopContext, Statement};
use go_parser::Token;
use go_parser::ast::{BranchStmt, CaseClause, EmptyStmt, Stmt};
use go_parser::AstObjects;
use std::collections::HashMap;
use std::rc::Rc;

use super::utils::{push_single, connect_if_empty};

impl ControlFlowGraph {
    /// Handle if statements
    #[allow(dead_code)]
    pub(crate) fn handle_if_statement(
        stmt: &Stmt,
        objs: &AstObjects,
        blocks: &mut HashMap<usize, BasicBlock>,
        next_id: &mut usize,
        prev_id: usize,
        exit_id: usize,
        current_loop_cond: Option<usize>,
        current_loop_after: Option<usize>,
        label_map: &mut HashMap<String, usize>,
        pending_gotos: &mut Vec<(usize, String)>,
    ) -> usize {
        if let Stmt::If(ifst) = stmt {
            let cond_id = *next_id;
            *next_id += 1;
            push_single(blocks, cond_id, stmt.clone());

            let mut then_label_map = label_map.clone();
            let (then_entry, then_exit, mut then_gotos) = Self::build_chain(
                objs,
                blocks,
                next_id,
                &ifst.body.list,
                exit_id,
                current_loop_cond,
                current_loop_after,
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
                        current_loop_cond,
                        current_loop_after,
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

            connect_if_empty!(prev_id, cond_id, blocks);

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

    /// Handle switch statements
    #[allow(dead_code)]
    pub(crate) fn handle_switch_statement(
        stmt: &Stmt,
        objs: &AstObjects,
        blocks: &mut HashMap<usize, BasicBlock>,
        next_id: &mut usize,
        prev_id: usize,
        exit_id: usize,
        current_loop_cond: Option<usize>,
        current_loop_after: Option<usize>,
        label_map: &mut HashMap<String, usize>,
        pending_gotos: &mut Vec<(usize, String)>,
    ) -> usize {
        if let Stmt::Switch(swtch) = stmt {
            let sw_id = *next_id;
            *next_id += 1;
            push_single(blocks, sw_id, stmt.clone());

            let mut case_info = Vec::new();

            for case_stmt in &swtch.body.list {
                if let Stmt::Case(case_rc) = case_stmt {
                    let case: &CaseClause = case_rc.as_ref();
                    let mut case_label_map = label_map.clone();
                    let (entry, exit, mut case_gotos) = Self::build_chain(
                        objs,
                        blocks,
                        next_id,
                        &case.body,
                        exit_id,
                        current_loop_cond,
                        current_loop_after,
                        &mut case_label_map,
                    );
                    for (k, v) in case_label_map {
                        label_map.insert(k, v);
                    }
                    pending_gotos.append(&mut case_gotos);

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

            connect_if_empty!(prev_id, sw_id, blocks);

            {
                if let Some(sw_block) = blocks.get_mut(&sw_id) {
                    for (entry, _, _) in &case_info {
                        sw_block.succs.push(*entry);
                    }
                    if sw_block.succs.is_empty() {
                        sw_block.succs.push(after_id);
                    }
                }
            }

            for i in 0..case_info.len() {
                let (_, exit, has_fallthrough) = case_info[i];
                if let Some(exit_block) = blocks.get_mut(&exit) {
                    if exit_block.succs.is_empty() {
                        if has_fallthrough && i + 1 < case_info.len() {
                            let next_entry = case_info[i + 1].0;
                            exit_block.succs = vec![next_entry];
                        } else {
                            exit_block.succs = vec![after_id];
                        }
                    }
                }
            }

            after_id
        } else {
            prev_id
        }
    }

    /// Handle branch statements (break, continue, goto, fallthrough)
    #[allow(dead_code)]
    pub(crate) fn handle_branch_statement(
        stmt: &Stmt,
        objs: &AstObjects,
        blocks: &mut HashMap<usize, BasicBlock>,
        next_id: &mut usize,
        prev_id: usize,
        exit_id: usize,
        current_loop_cond: Option<usize>,
        current_loop_after: Option<usize>,
        loop_contexts: &HashMap<String, LoopContext>,
        label_map: &HashMap<String, usize>,
        pending_gotos: &mut Vec<(usize, String)>,
    ) -> usize {
        if let Stmt::Branch(br_rc) = stmt {
            let id = *next_id;
            *next_id += 1;
            push_single(blocks, id, stmt.clone());

            connect_if_empty!(prev_id, id, blocks);

            let bs: &BranchStmt = br_rc.as_ref();

            if let Some(branch_block) = blocks.get_mut(&id) {
                match bs.token {
                    Token::GOTO => {
                        if let Some(label_ident) = bs.label {
                            let name = objs.idents[label_ident].name.clone();
                            if let Some(&target) = label_map.get(&name) {
                                branch_block.succs = vec![target];
                            } else {
                                pending_gotos.push((id, name));
                            }
                        }
                    }
                    Token::BREAK => {
                        if let Some(label_ident) = bs.label {
                            let name = objs.idents[label_ident].name.clone();
                            if let Some(context) = loop_contexts.get(&name) {
                                branch_block.succs = vec![context.after_id];
                            } else if let Some(&target) = label_map.get(&name) {
                                branch_block.succs = vec![target];
                            } else {
                                pending_gotos.push((id, name));
                            }
                        } else if let Some(after) = current_loop_after {
                            branch_block.succs = vec![after];
                        } else {
                            branch_block.succs = vec![exit_id];
                        }
                    }
                    Token::CONTINUE => {
                        if let Some(label_ident) = bs.label {
                            let name = objs.idents[label_ident].name.clone();
                            if let Some(context) = loop_contexts.get(&name) {
                                branch_block.succs = vec![context.cond_id];
                            } else if let Some(&target) = label_map.get(&name) {
                                // Fallback for labels that aren't loop contexts
                                branch_block.succs = vec![target];
                            } else {
                                pending_gotos.push((id, name));
                            }
                        } else if let Some(cond) = current_loop_cond {
                            branch_block.succs = vec![cond];
                        } else {
                            branch_block.succs = vec![exit_id];
                        }
                    }
                    Token::FALLTHROUGH => {}
                    _ => {}
                }
            }

            id
        } else {
            prev_id
        }
    }

    /// Handle return statements
    #[allow(dead_code)]
    pub(crate) fn handle_return_statement(
        stmt: &Stmt,
        blocks: &mut HashMap<usize, BasicBlock>,
        next_id: &mut usize,
        prev_id: usize,
        exit_id: usize,
    ) -> usize {
        if let Stmt::Return(_) = stmt {
            let id = *next_id;
            *next_id += 1;
            push_single(blocks, id, stmt.clone());

            connect_if_empty!(prev_id, id, blocks);

            if let Some(return_block) = blocks.get_mut(&id) {
                return_block.succs = vec![exit_id];
            }

            id
        } else {
            prev_id
        }
    }
}

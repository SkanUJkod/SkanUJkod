use crate::cfg::types::{BasicBlock, ControlFlowGraph, LoopContext, Statement};
use go_parser::Token;
use go_parser::ast::{BlockStmt, BranchStmt, CaseClause, Decl, EmptyStmt, FuncDecl, Node, Stmt};
use go_parser::{AstObjects, FileSet};
use std::collections::HashMap;
use std::rc::Rc;

fn get_precise_line_info(fset: &FileSet, objs: &AstObjects, stmt: &Stmt) -> (usize, usize) {
    let pos = stmt.pos(objs);
    let end_pos = stmt.end(objs);

    let start_line = fset.position(pos).map(|p| p.line).unwrap_or(0);
    let end_line = fset.position(end_pos).map(|p| p.line).unwrap_or(start_line);

    (start_line, end_line)
}

impl ControlFlowGraph {
    pub fn build(fset: &FileSet, func: &FuncDecl, objs: &AstObjects) -> Self {
        let mut blocks = HashMap::<usize, BasicBlock>::new();
        let mut next_id = 0;
        let entry = next_id;
        next_id += 1;

        let func_start_line = fset.position(func.pos(objs)).map(|p| p.line).unwrap_or(0);

        blocks.insert(
            entry,
            BasicBlock {
                id: entry,
                stmts: Vec::new(),
                succs: Vec::new(),
                start_line: func_start_line,
                end_line: func_start_line,
            },
        );

        let exit_id = next_id;
        next_id += 1;

        let func_end_line = if let Some(body) = &func.body {
            fset.position(body.end()).map(|p| p.line).unwrap_or(func_start_line)
        } else {
            func_start_line
        };

        blocks.insert(
            exit_id,
            BasicBlock {
                id: exit_id,
                stmts: vec![],
                succs: vec![],
                start_line: func_end_line,
                end_line: func_end_line,
            },
        );

        let body_rc = match &func.body {
            Some(b) => b,
            None => {
                if let Some(entry_block) = blocks.get_mut(&entry) {
                    entry_block.succs = vec![exit_id];
                }
                return ControlFlowGraph { blocks, entry };
            }
        };
        let body: &BlockStmt = body_rc.as_ref();

        let push_single = |blocks: &mut HashMap<usize, BasicBlock>, id: usize, stmt: Stmt| {
            let (start_line, end_line) = get_precise_line_info(fset, objs, &stmt);
            let stmt_text = format!("{:?}", stmt);
            blocks.insert(
                id,
                BasicBlock {
                    id,
                    stmts: vec![crate::cfg::types::Statement {
                        text: stmt_text,
                        stmt,
                    }],
                    succs: Vec::new(),
                    start_line,
                    end_line,
                },
            );
        };

        let mut prev_id = entry;
        macro_rules! connect_if_empty {
            ($from:expr, $to:expr, $blocks:expr) => {{
                if let Some(b) = $blocks.get_mut(&$from) {
                    if b.succs.is_empty() {
                        b.succs = vec![$to];
                    }
                }
            }};
        }
        let mut label_map: HashMap<String, usize> = HashMap::new();
        let mut loop_contexts: HashMap<String, LoopContext> = HashMap::new();
        let mut label_def_ids: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut pending_gotos: Vec<(usize, String)> = Vec::new();
        let current_loop_cond: Option<usize> = None;
        let current_loop_after: Option<usize> = None;

        for stmt in &body.list {
            match stmt {
                Stmt::If(ifst) => {
                    let cond_id = next_id;
                    next_id += 1;
                    push_single(&mut blocks, cond_id, stmt.clone());

                    let mut then_label_map = label_map.clone();
                    let (then_entry, then_exit, mut then_gotos) = Self::build_chain(
                        objs,
                        &mut blocks,
                        &mut next_id,
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
                                &mut blocks,
                                &mut next_id,
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

                    let after_id = next_id;
                    next_id += 1;
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

                    if let Some(prev_block) = blocks.get_mut(&prev_id) {
                        prev_block.succs = vec![cond_id];
                    }

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

                    prev_id = after_id;
                }
                Stmt::For(forst) => {
                    let init_id = if let Some(init_stmt) = &forst.init {
                        let id = next_id;
                        next_id += 1;
                        push_single(&mut blocks, id, init_stmt.clone());

                        if let Some(prev_block) = blocks.get_mut(&prev_id) {
                            prev_block.succs = vec![id];
                        }

                        id
                    } else {
                        prev_id
                    };

                    let cond_id = next_id;
                    next_id += 1;
                    let cond_stmt = if let Some(cond_expr) = &forst.cond {
                        Stmt::Expr(Box::new(cond_expr.clone()))
                    } else {
                        Stmt::Empty(Rc::new(EmptyStmt {
                            semi: 0,
                            implicit: true,
                        }))
                    };
                    push_single(&mut blocks, cond_id, cond_stmt);

                    if init_id == prev_id {
                        if let Some(prev_block) = blocks.get_mut(&prev_id) {
                            prev_block.succs = vec![cond_id];
                        }
                    } else {
                        if let Some(init_block) = blocks.get_mut(&init_id) {
                            init_block.succs = vec![cond_id];
                        }
                    }

                    let after_id = next_id;
                    next_id += 1;
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
                        &mut blocks,
                        &mut next_id,
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
                        let post_id = next_id;
                        next_id += 1;
                        push_single(&mut blocks, post_id, post_stmt.clone());

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

                    prev_id = after_id;
                }
                Stmt::Range(rng) => {
                    let cond_id = next_id;
                    next_id += 1;
                    push_single(&mut blocks, cond_id, stmt.clone());

                    let after_id = next_id;
                    next_id += 1;
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
                        &mut blocks,
                        &mut next_id,
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

                    if let Some(prev_block) = blocks.get_mut(&prev_id) {
                        prev_block.succs = vec![cond_id];
                    }

                    if let Some(cond_block) = blocks.get_mut(&cond_id) {
                        cond_block.succs.push(body_entry);
                        cond_block.succs.push(after_id);
                    }

                    prev_id = after_id;
                }
                Stmt::Switch(swtch) => {
                    let sw_id = next_id;
                    next_id += 1;
                    push_single(&mut blocks, sw_id, stmt.clone());

                    let mut case_info = Vec::new();

                    for case_stmt in &swtch.body.list {
                        if let Stmt::Case(case_rc) = case_stmt {
                            let case: &CaseClause = case_rc.as_ref();
                            let mut case_label_map = label_map.clone();
                            let (entry, exit, mut case_gotos) = Self::build_chain(
                                objs,
                                &mut blocks,
                                &mut next_id,
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

                    let after_id = next_id;
                    next_id += 1;
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

                    if let Some(prev_block) = blocks.get_mut(&prev_id) {
                        prev_block.succs = vec![sw_id];
                    }

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

                    prev_id = after_id;
                }
                Stmt::Labeled(lbl_key) => {
                    let id = next_id;
                    next_id += 1;
                    push_single(&mut blocks, id, stmt.clone());
                    label_def_ids.insert(id);
                    connect_if_empty!(prev_id, id, blocks);

                    let lbl = &objs.l_stmts[*lbl_key];
                    let name = objs.idents[lbl.label].name.clone();
                    label_map.insert(name.clone(), id);

                    let inner_stmt = lbl.stmt.clone();
                    match inner_stmt {
                        Stmt::For(forst) => {
                            let prev_saved = prev_id;
                            prev_id = id;

                            let init_id = if let Some(init_stmt) = &forst.init {
                                let fid = next_id;
                                next_id += 1;
                                push_single(&mut blocks, fid, init_stmt.clone());
                                connect_if_empty!(prev_id, fid, blocks);
                                fid
                            } else {
                                prev_id
                            };

                            let cond_id = next_id;
                            next_id += 1;
                            let cond_stmt = if let Some(cond_expr) = &forst.cond {
                                Stmt::Expr(Box::new(cond_expr.clone()))
                            } else {
                                Stmt::Empty(Rc::new(EmptyStmt {
                                    semi: 0,
                                    implicit: true,
                                }))
                            };
                            push_single(&mut blocks, cond_id, cond_stmt);
                            if init_id == prev_id {
                                connect_if_empty!(prev_id, cond_id, blocks);
                            } else {
                                connect_if_empty!(init_id, cond_id, blocks);
                            }

                            let after_id = next_id;
                            next_id += 1;
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
                                &mut blocks,
                                &mut next_id,
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
                                let post_id = next_id;
                                next_id += 1;
                                push_single(&mut blocks, post_id, post_stmt.clone());

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

                            prev_id = after_id;
                            let _ = prev_saved;
                        }
                        Stmt::Range(_) => {
                            let inner_vec = vec![inner_stmt.clone()];
                            let (entry, exit, mut rng_gotos) = Self::build_chain(
                                objs,
                                &mut blocks,
                                &mut next_id,
                                &inner_vec,
                                exit_id,
                                current_loop_cond,
                                current_loop_after,
                                &mut label_map,
                            );
                            pending_gotos.append(&mut rng_gotos);

                            if let Some(lb) = blocks.get_mut(&id) {
                                if lb.succs.is_empty() {
                                    lb.succs = vec![entry];
                                }
                            }
                            prev_id = exit;
                        }
                        _ => {
                            prev_id = id;
                        }
                    }
                }
                Stmt::Branch(br_rc) => {
                    let id = next_id;
                    next_id += 1;
                    push_single(&mut blocks, id, stmt.clone());

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

                    prev_id = id;
                }
                Stmt::Return(_) => {
                    let id = next_id;
                    next_id += 1;
                    push_single(&mut blocks, id, stmt.clone());

                    connect_if_empty!(prev_id, id, blocks);

                    if let Some(return_block) = blocks.get_mut(&id) {
                        return_block.succs = vec![exit_id];
                    }

                    prev_id = id;
                }
                _ => {
                    let id = next_id;
                    next_id += 1;
                    push_single(&mut blocks, id, stmt.clone());

                    connect_if_empty!(prev_id, id, blocks);

                    prev_id = id;
                }
            }
        }

        // Resolve pending gotos
        for (blk_id, label_name) in pending_gotos {
            if let Some(&target) = label_map.get(&label_name) {
                if let Some(goto_block) = blocks.get_mut(&blk_id) {
                    goto_block.succs = vec![target];
                }
            } else {
                eprintln!(
                    "Warning: unresolved label '{}' in block {}",
                    label_name, blk_id
                );
                if let Some(goto_block) = blocks.get_mut(&blk_id) {
                    goto_block.succs = vec![exit_id];
                }
            }
        }

        if prev_id != exit_id {
            if let Some(prev_block) = blocks.get_mut(&prev_id) {
                if prev_block.succs.is_empty() {
                    prev_block.succs.push(exit_id);
                }
            }
        }

        crate::cfg::optimization::clean_unreachable_blocks(&mut blocks, entry);
        crate::cfg::optimization::optimize_empty_blocks(
            &mut blocks,
            entry,
            exit_id,
            &label_def_ids,
        );
        crate::cfg::optimization::clean_unreachable_blocks(&mut blocks, entry);
        crate::cfg::validation::validate_graph(&blocks, exit_id);

        if let Some(exit_blk) = blocks.get_mut(&exit_id) {
            exit_blk.succs.clear();
        }

        ControlFlowGraph { blocks, entry }
    }

    fn build_chain(
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
            return (id, id, Vec::new());
        }

        let mut prev: Option<usize> = None;
        let entry = *next_id;
        let mut last_id = entry;
        let mut pending_gotos: Vec<(usize, String)> = Vec::new();

        for stmt in stmts {
            match stmt {
                Stmt::If(ifst) => {
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

                    if let Some(p) = prev {
                        if let Some(prev_block) = blocks.get(&p) {
                            if prev_block.succs.is_empty() {
                                if let Some(pb) = blocks.get_mut(&p) {
                                    pb.succs = vec![cond_id];
                                }
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

                    prev = Some(after_id);
                    last_id = after_id;
                }
                Stmt::For(forst) => {
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
                        if let Some(p) = prev {
                            if let Some(prev_block) = blocks.get(&p) {
                                if prev_block.succs.is_empty() {
                                    if let Some(pb) = blocks.get_mut(&p) {
                                        pb.succs = vec![id];
                                    }
                                }
                            }
                        }
                        id
                    } else if let Some(p) = prev {
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

                    prev = Some(after_id);
                    last_id = after_id;
                }
                Stmt::Range(rng) => {
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

                    if let Some(p) = prev {
                        if let Some(prev_block) = blocks.get(&p) {
                            if prev_block.succs.is_empty() {
                                if let Some(prev_block) = blocks.get_mut(&p) {
                                    prev_block.succs = vec![nested_cond_id];
                                }
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

                    prev = Some(nested_after_id);
                    last_id = nested_after_id;
                }
                Stmt::Labeled(lbl_key) => {
                    // Create a block for the label itself
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
                        if let Some(prev_block) = blocks.get(&p) {
                            if prev_block.succs.is_empty() {
                                if let Some(prev_block) = blocks.get_mut(&p) {
                                    prev_block.succs = vec![id];
                                }
                            }
                        }
                    }

                    let lbl = &objs.l_stmts[*lbl_key];
                    let name = objs.idents[lbl.label].name.clone();
                    label_map.insert(name.clone(), id);

                    let inner_stmt = lbl.stmt.clone();
                    match inner_stmt {
                        Stmt::For(_) | Stmt::Range(_) => {
                            let tmp_vec = vec![inner_stmt];
                            let (entry, exit, mut inner_gotos) = Self::build_chain(
                                objs, blocks, next_id, &tmp_vec, exit_id, loop_cond, loop_after,
                                label_map,
                            );
                            pending_gotos.append(&mut inner_gotos);
                            if let Some(lb) = blocks.get_mut(&id) {
                                if lb.succs.is_empty() {
                                    lb.succs = vec![entry];
                                }
                            }
                            prev = Some(exit);
                            last_id = exit;
                        }
                        _ => {
                            prev = Some(id);
                            last_id = id;
                        }
                    }
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
                        Stmt::Branch(br_rc) => {
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
}

pub fn build_cfgs_for_file(
    fset: &FileSet,
    objs: &AstObjects,
    file: &go_parser::ast::File,
) -> HashMap<String, ControlFlowGraph> {
    let mut out = HashMap::new();
    for decl in &file.decls {
        if let Decl::Func(fk) = *decl {
            let fdecl = &objs.fdecls[fk];
            let name = objs.idents[fdecl.name].name.clone();
            out.insert(name, ControlFlowGraph::build(fset, fdecl, objs));
        }
    }
    out
}

use std::fs::File;
use std::io::Write;

pub fn export_instrumented_go(
    func_name: &str,
    cfg: &ControlFlowGraph,
    output_path: &std::path::Path,
) -> std::io::Result<()> {
    let mut file = File::create(output_path)?;
    writeln!(file, "package main\n")?;
    writeln!(file, "var hits = map[string]map[int]bool{{}}")?;
    writeln!(
        file,
        "func stmt_hit(funcName string, id int) {{\n\tif _, ok := hits[funcName]; !ok {{\n\t\thits[funcName] = map[int]bool{{}}\n\t}}\n\thits[funcName][id] = true\n}}\n"
    )?;

    writeln!(file, "func {}() {{", func_name)?;

    for (block_id, block) in &cfg.blocks {
        for _stmt in &block.stmts {
            writeln!(file, "\tstmt_hit(\"{}\", {});", func_name, block_id)?;
        }
    }

    writeln!(file, "}}")?;
    Ok(())
}

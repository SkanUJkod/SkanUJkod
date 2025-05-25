//! Control Flow Graph with top-level divisions: 4 blocks for each `if` (cond/then/else/after)
use go_parser::ast::{BlockStmt, Decl, FuncDecl, Stmt, CaseClause, EmptyStmt, BranchStmt};
use go_parser::Token;
use go_parser::{AstObjects, FileSet};
use std::collections::HashMap;
use std::rc::Rc;

/// Single basic block
#[derive(Debug, Clone)]
pub struct BasicBlock {
    #[allow(dead_code)]
    pub id: usize,
    pub stmts: Vec<Stmt>,
    pub succs: Vec<usize>,
}

/// Control flow graph for a function
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub blocks: HashMap<usize, BasicBlock>,
    pub entry: usize,
}

impl ControlFlowGraph {
    pub fn build(func: &FuncDecl, objs: &AstObjects) -> Self {
        let mut blocks = HashMap::<usize, BasicBlock>::new();
        let mut next_id = 0;
        let entry = next_id;
        next_id += 1;
        blocks.insert(entry, BasicBlock { id: entry, stmts: Vec::new(), succs: Vec::new() });

        let exit_id = next_id;
        next_id += 1;
        blocks.insert(exit_id, BasicBlock { id: exit_id, stmts: vec![], succs: vec![] });

        let body_rc = match &func.body {
            Some(b) => b,
            None => {
                blocks.get_mut(&entry).unwrap().succs = vec![exit_id];
                return ControlFlowGraph { blocks, entry };
            }
        };
        let body: &BlockStmt = body_rc.as_ref();

        // Helper: creates block with a single stmt
        let push_single = |blocks: &mut HashMap<usize, BasicBlock>, id: usize, stmt: Stmt| {
            blocks.insert(id, BasicBlock { id, stmts: vec![stmt], succs: vec![] });
        };

        let mut prev_id = entry;
        // Label map (Label → block id) and pending gotos to resolve
        let mut label_map: HashMap<String, usize> = HashMap::new();
        let mut pending_gotos: Vec<(usize, String)> = Vec::new();
        
        for stmt in &body.list {
            match stmt {
                Stmt::If(ifst) => {
                    // Condition block
                    let cond_id = next_id; next_id += 1;
                    push_single(&mut blocks, cond_id, stmt.clone());

                    // THEN branch
                    let mut then_label_map = label_map.clone();
                    let (then_entry, then_exit, mut then_gotos) = Self::build_chain(
                        objs, &mut blocks, &mut next_id, &ifst.body.list, exit_id, None, None, &mut then_label_map
                    );
                    // Update main label map with any labels defined in then branch
                    for (k, v) in then_label_map {
                        label_map.insert(k, v);
                    }
                    pending_gotos.append(&mut then_gotos);

                    // ELSE branch
                    let else_result = if let Some(els_stmt) = &ifst.els {
                        let list: &[Stmt] = match els_stmt {
                            Stmt::Block(bs) => &bs.list,
                            Stmt::If(if_nested) => &if_nested.body.list,
                            _ => &[],
                        };
                        if list.is_empty() { None }
                        else {
                            let mut else_label_map = label_map.clone();
                            let (entry, exit, mut else_gotos) = Self::build_chain(
                                objs, &mut blocks, &mut next_id, list, exit_id, None, None, &mut else_label_map
                            );
                            // Update main label map with any labels defined in else branch
                            for (k, v) in else_label_map {
                                label_map.insert(k, v);
                            }
                            pending_gotos.append(&mut else_gotos);
                            Some((entry, exit))
                        }
                    } else {
                        None
                    };

                    // AFTER block - merge point
                    let after_id = next_id;
                    next_id += 1;
                    blocks.insert(
                        after_id,
                        BasicBlock {
                            id: after_id,
                            stmts: vec![Stmt::Empty(Rc::new(EmptyStmt { semi: 0, implicit: true }))],
                            succs: vec![exit_id],
                        },
                    );

                    // Connect edges
                    blocks.get_mut(&prev_id).unwrap().succs = vec![cond_id];
                    let cond_succs = &mut blocks.get_mut(&cond_id).unwrap().succs;
                    cond_succs.push(then_entry);
                    cond_succs.push(else_result.map(|(entry, _)| entry).unwrap_or(after_id));
                    
                    if blocks.get(&then_exit).unwrap().succs.is_empty() {
                        blocks.get_mut(&then_exit).unwrap().succs = vec![after_id];
                    }
                    
                    if let Some((_, else_exit)) = else_result {
                        if blocks.get(&else_exit).unwrap().succs.is_empty() {
                            blocks.get_mut(&else_exit).unwrap().succs = vec![after_id];
                        }
                    }
                    
                    prev_id = after_id;
                }
                Stmt::For(forst) => {
                    // Init block
                    let init_id = if let Some(init_stmt) = &forst.init {
                        let id = next_id; next_id += 1;
                        push_single(&mut blocks, id, init_stmt.clone());
                        blocks.get_mut(&prev_id).unwrap().succs = vec![id];
                        id
                    } else {
                        prev_id
                    };

                    // Condition block
                    let cond_id = next_id; next_id += 1;
                    let cond_stmt = if let Some(cond_expr) = &forst.cond {
                        Stmt::Expr(Box::new(cond_expr.clone()))
                    } else {
                        // Unconditional loop: `for { … }`
                        Stmt::Empty(Rc::new(EmptyStmt { semi: 0, implicit: true }))
                    };
                    push_single(&mut blocks, cond_id, cond_stmt);

                    if init_id == prev_id {
                        blocks.get_mut(&prev_id).unwrap().succs = vec![cond_id];
                    } else {
                        blocks.get_mut(&init_id).unwrap().succs = vec![cond_id];
                    }

                    // After block
                    let after_id = next_id; next_id += 1;
                    blocks.insert(after_id, BasicBlock { id: after_id, stmts: vec![], succs: vec![exit_id] });

                    // Body chain
                    let mut loop_label_map = label_map.clone();
                    let (body_entry, body_exit, mut body_gotos) = Self::build_chain(
                        objs,
                        &mut blocks,
                        &mut next_id,
                        &forst.body.list,
                        exit_id,
                        Some(cond_id),   // continue → cond
                        Some(after_id),  // break    → after
                        &mut loop_label_map
                    );
                    // Update main label map
                    for (k, v) in loop_label_map {
                        label_map.insert(k, v);
                    }
                    pending_gotos.append(&mut body_gotos);

                    // Post block
                    if let Some(post_stmt) = &forst.post {
                        let post_id = next_id; next_id += 1;
                        push_single(&mut blocks, post_id, post_stmt.clone());

                        // body_exit → post
                        if blocks.get(&body_exit).unwrap().succs.is_empty() {
                            blocks.get_mut(&body_exit).unwrap().succs = vec![post_id];
                        }
                        // post → cond
                        blocks.get_mut(&post_id).unwrap().succs = vec![cond_id];
                    } else if blocks.get(&body_exit).unwrap().succs.is_empty() {
                        // No post: body → cond
                        blocks.get_mut(&body_exit).unwrap().succs = vec![cond_id];
                    }

                    // Connect cond edges
                    let cond_succs = &mut blocks.get_mut(&cond_id).unwrap().succs;
                    cond_succs.push(body_entry);
                    cond_succs.push(after_id);

                    prev_id = after_id;
                }
                Stmt::Range(rng) => {
                    // Condition block / next element from range
                    let cond_id = next_id; next_id += 1;
                    push_single(&mut blocks, cond_id, stmt.clone());

                    // After block (loop exit)
                    let after_id = next_id; next_id += 1;
                    blocks.insert(after_id, BasicBlock { id: after_id, stmts: vec![], succs: vec![exit_id] });

                    // Loop body
                    let mut range_label_map = label_map.clone();
                    let (body_entry, body_exit, mut range_gotos) = Self::build_chain(
                        objs,
                        &mut blocks,
                        &mut next_id,
                        &rng.body.list,
                        exit_id,
                        Some(cond_id),   // continue
                        Some(after_id),  // break
                        &mut range_label_map
                    );
                    // Update main label map
                    for (k, v) in range_label_map {
                        label_map.insert(k, v);
                    }
                    pending_gotos.append(&mut range_gotos);

                    // If body doesn't have an explicit jump, go back to condition
                    if blocks.get(&body_exit).unwrap().succs.is_empty() {
                        blocks.get_mut(&body_exit).unwrap().succs = vec![cond_id];
                    }

                    // Connect edges
                    blocks.get_mut(&prev_id).unwrap().succs = vec![cond_id];
                    let cond_succs = &mut blocks.get_mut(&cond_id).unwrap().succs;
                    cond_succs.push(body_entry);
                    cond_succs.push(after_id);

                    prev_id = after_id;
                }
                Stmt::Switch(swtch) => {
                    // Switch block
                    let sw_id = next_id; next_id += 1;
                    push_single(&mut blocks, sw_id, stmt.clone());

                    // Build each case clause as a chain
                    let mut case_exits = Vec::new();
                    for case_stmt in &swtch.body.list {
                        if let Stmt::Case(case_rc) = case_stmt {
                            let case: &CaseClause = case_rc.as_ref();
                            let mut case_label_map = label_map.clone();
                            let (entry, exit, mut case_gotos) = Self::build_chain(
                                objs, &mut blocks, &mut next_id, &case.body, exit_id, None, None, &mut case_label_map
                            );
                            // Update main label map
                            for (k, v) in case_label_map {
                                label_map.insert(k, v);
                            }
                            pending_gotos.append(&mut case_gotos);
                            case_exits.push((entry, exit));
                        }
                    }

                    // After-switch block
                    let after_id = next_id; next_id += 1;
                    blocks.insert(after_id, BasicBlock { id: after_id, stmts: Vec::new(), succs: vec![exit_id] });

                    // Connect previous block to switch
                    blocks.get_mut(&prev_id).unwrap().succs = vec![sw_id];
                    
                    // Connect switch to each case entry
                    {
                        let sw_succs = &mut blocks.get_mut(&sw_id).unwrap().succs;
                        for (entry, _) in &case_exits {
                            sw_succs.push(*entry);
                        }
                        // If no cases, link directly to after block
                        if sw_succs.is_empty() {
                            sw_succs.push(after_id);
                        }
                    }
                    
                    // Handle case fall-through: connect each case to the next one if not explicit jump
                    let mut prev_case_exit = None;
                    for (entry, exit) in &case_exits {
                        if let Some(prev_exit) = prev_case_exit {
                            // Add fall-through edge if no explicit jump
                            if blocks.get(&prev_exit).unwrap().succs.is_empty() {
                                blocks.get_mut(&prev_exit).unwrap().succs = vec![*entry];
                            }
                        }
                        prev_case_exit = Some(*exit);
                    }
                    
                    // Last case exit goes to after_id if no explicit jump
                    if let Some(last_exit) = prev_case_exit {
                        if blocks.get(&last_exit).unwrap().succs.is_empty() {
                            blocks.get_mut(&last_exit).unwrap().succs = vec![after_id];
                        }
                    }

                    prev_id = after_id;
                }
                Stmt::Labeled(lbl_key) => {
                    // New block for the label
                    let id = next_id; next_id += 1;
                    push_single(&mut blocks, id, stmt.clone());
                    // Connect previous → label
                    blocks.get_mut(&prev_id).unwrap().succs = vec![id];
                    // Register label
                    let lbl = &objs.l_stmts[*lbl_key];
                    let name = objs.idents[lbl.label].name.clone();
                    label_map.insert(name, id);
                    prev_id = id;
                }
                Stmt::Branch(br_rc) => {
                    let id = next_id; next_id += 1;
                    push_single(&mut blocks, id, stmt.clone());
                    blocks.get_mut(&prev_id).unwrap().succs = vec![id];
                    
                    // Handle branch statements
                    let bs: &BranchStmt = br_rc.as_ref();
                    match bs.token {
                        Token::GOTO => {
                            if let Some(label_ident) = bs.label {
                                let name = objs.idents[label_ident].name.clone();
                                if let Some(&target) = label_map.get(&name) {
                                    // Label already defined
                                    blocks.get_mut(&id).unwrap().succs = vec![target];
                                } else {
                                    // Label not yet defined, add to pending
                                    pending_gotos.push((id, name));
                                }
                            }
                        },
                        Token::BREAK => {
                            // Check if break has a label
                            if let Some(label_ident) = bs.label {
                                let name = objs.idents[label_ident].name.clone();
                                if let Some(&target) = label_map.get(&name) {
                                    // Labeled break
                                    blocks.get_mut(&id).unwrap().succs = vec![target];
                                } else {
                                    // Unresolved labeled break
                                    pending_gotos.push((id, name));
                                }
                            } else {
                                // Normal break outside loop goes to exit
                                blocks.get_mut(&id).unwrap().succs = vec![exit_id];
                            }
                        },
                        Token::CONTINUE => {
                            // Handle outside-of-loop continue as exit
                            blocks.get_mut(&id).unwrap().succs = vec![exit_id];
                        },
                        _ => {}
                    }
                    
                    prev_id = id;
                }
                Stmt::Return(_) => {
                    let id = next_id; next_id += 1;
                    push_single(&mut blocks, id, stmt.clone());
                    blocks.get_mut(&prev_id).unwrap().succs = vec![id];
                    blocks.get_mut(&id).unwrap().succs = vec![exit_id];
                    prev_id = exit_id;
                }
                _ => {
                    let id = next_id; next_id += 1;
                    push_single(&mut blocks, id, stmt.clone());
                    blocks.get_mut(&prev_id).unwrap().succs = vec![id];
                    prev_id = id;
                }
            }
        }

        // Resolve pending gotos
        for (blk_id, label_name) in pending_gotos {
            if let Some(&target) = label_map.get(&label_name) {
                blocks.get_mut(&blk_id).unwrap().succs = vec![target];
            } else {
                // Label not found, treat as exit
                eprintln!("Warning: unresolved goto label: {}", label_name);
                blocks.get_mut(&blk_id).unwrap().succs = vec![exit_id];
            }
        }

        // If last block doesn't have an explicit jump, connect to exit
        if prev_id != exit_id {
            blocks.get_mut(&prev_id).unwrap().succs.push(exit_id);
        }

        // Prune unreachable blocks
        {
            use std::collections::HashSet;
            let mut visited = HashSet::new();
            let mut stack = vec![entry];
            while let Some(id) = stack.pop() {
                if !visited.insert(id) {
                    continue;
                }
                if let Some(block) = blocks.get(&id) {
                    for &succ in &block.succs {
                        stack.push(succ);
                    }
                }
            }
            blocks.retain(|id, _| visited.contains(id));
        }

        // Collapse trivial "Empty" blocks
        {
            let mut changed = true;
            while changed {
                changed = false;
                // First collect candidate (rid, succ) pairs
                let mut collapse_pairs = Vec::new();
                for (bid, blk) in &blocks {
                    if *bid == entry {
                        continue; // never remove the entry block
                    }
                    if blk.stmts.len() == 1 {
                        if let Stmt::Empty(_) = &blk.stmts[0] {
                            if blk.succs.len() == 1 {
                                let succ = blk.succs[0];
                                if succ != *bid && blocks.contains_key(&succ) {
                                    collapse_pairs.push((*bid, succ));
                                }
                            }
                        }
                    }
                }
                if !collapse_pairs.is_empty() {
                    changed = true;
                    for (rid, succ) in collapse_pairs {
                        // Redirect all predecessors
                        for (_other_id, other_blk) in blocks.iter_mut() {
                            for s in &mut other_blk.succs {
                                if *s == rid {
                                    *s = succ;
                                }
                            }
                        }
                        blocks.remove(&rid);
                    }
                }
            }
        }

        // Prune unreachable blocks again after collapsing empties
        {
            use std::collections::HashSet;
            let mut visited = HashSet::new();
            let mut stack = vec![entry];
            while let Some(id) = stack.pop() {
                if !visited.insert(id) {
                    continue;
                }
                if let Some(block) = blocks.get(&id) {
                    for &succ in &block.succs {
                        stack.push(succ);
                    }
                }
            }
            blocks.retain(|id, _| visited.contains(id));
        }

        // Ensure exit block has no successors
        if let Some(exit_blk) = blocks.get_mut(&exit_id) {
            exit_blk.succs.clear();
        }
        ControlFlowGraph { blocks, entry }
    }

    /// Builds a linear chain of blocks from `stmts`; returns (entry_id, exit_id, pending_gotos)
    fn build_chain(
        objs: &AstObjects,
        blocks: &mut HashMap<usize, BasicBlock>,
        next_id: &mut usize,
        stmts: &[Stmt],
        exit_id: usize,
        loop_cond: Option<usize>,
        loop_after: Option<usize>,
        label_map: &mut HashMap<String, usize>,  // Now mutable
    ) -> (usize, usize, Vec<(usize, String)>) {
        if stmts.is_empty() {
            let id = *next_id; *next_id += 1;
            blocks.insert(id, BasicBlock { 
                id, 
                stmts: vec![Stmt::Empty(Rc::new(EmptyStmt { semi: 0, implicit: true }))], 
                succs: vec![] 
            });
            return (id, id, Vec::new());
        }

        let mut prev: Option<usize> = None;
        let entry = *next_id;
        let mut last_id = entry;
        let mut pending_gotos: Vec<(usize, String)> = Vec::new();

        for stmt in stmts {
            match stmt {
                Stmt::If(ifst) => {
                    // Condition block
                    let cond_id = *next_id; *next_id += 1;
                    blocks.insert(cond_id, BasicBlock { id: cond_id, stmts: vec![stmt.clone()], succs: vec![] });
                    if let Some(p) = prev {
                        if blocks.get(&p).unwrap().succs.is_empty() {
                            blocks.get_mut(&p).unwrap().succs = vec![cond_id];
                        }
                    }
                    
                    // Process then branch
                    let mut then_label_map = label_map.clone();
                    let (then_entry, then_exit, mut then_gotos) = Self::build_chain(
                        objs, blocks, next_id, &ifst.body.list, exit_id, loop_cond, loop_after, &mut then_label_map
                    );
                    // Update label map
                    for (k, v) in then_label_map {
                        label_map.insert(k, v);
                    }
                    pending_gotos.append(&mut then_gotos);

                    // Process else branch
                    let else_result = if let Some(els_stmt) = &ifst.els {
                        let list: &[Stmt] = match els_stmt {
                            Stmt::Block(bs) => &bs.list,
                            Stmt::If(if_nested) => &if_nested.body.list,
                            _ => &[],
                        };
                        if list.is_empty() { None }
                        else {
                            let mut else_label_map = label_map.clone();
                            let (entry, exit, mut else_gotos) = Self::build_chain(
                                objs, blocks, next_id, list, exit_id, loop_cond, loop_after, &mut else_label_map
                            );
                            // Update label map
                            for (k, v) in else_label_map {
                                label_map.insert(k, v);
                            }
                            pending_gotos.append(&mut else_gotos);
                            Some((entry, exit))
                        }
                    } else {
                        None
                    };

                    // After block (merge point)
                    let after_id = *next_id; *next_id += 1;
                    blocks.insert(
                        after_id,
                        BasicBlock {
                            id: after_id,
                            stmts: vec![Stmt::Empty(Rc::new(EmptyStmt { semi: 0, implicit: true }))],
                            succs: vec![],
                        },
                    );
                    
                    // Connect edges
                    let cond_succs = &mut blocks.get_mut(&cond_id).unwrap().succs;
                    cond_succs.push(then_entry);
                    cond_succs.push(else_result.map(|(entry, _)| entry).unwrap_or(after_id));
                    if blocks.get(&then_exit).unwrap().succs.is_empty() {
                        blocks.get_mut(&then_exit).unwrap().succs = vec![after_id];
                    }
                    if let Some((_, else_exit)) = else_result {
                        if blocks.get(&else_exit).unwrap().succs.is_empty() {
                            blocks.get_mut(&else_exit).unwrap().succs = vec![after_id];
                        }
                    }
                    prev = Some(after_id);
                    last_id = after_id;
                }
                Stmt::Labeled(lbl_key) => {
                    // New block for the label
                    let id = *next_id; *next_id += 1;
                    blocks.insert(id, BasicBlock { id, stmts: vec![stmt.clone()], succs: vec![] });
                    
                    // Connect previous → label
                    if let Some(p) = prev {
                        if blocks.get(&p).unwrap().succs.is_empty() {
                            blocks.get_mut(&p).unwrap().succs = vec![id];
                        }
                    }
                    
                    // Register label
                    let lbl = &objs.l_stmts[*lbl_key];
                    let name = objs.idents[lbl.label].name.clone();
                    label_map.insert(name, id);
                    
                    prev = Some(id);
                    last_id = id;
                }
                _ => {
                    let id = *next_id; *next_id += 1;
                    last_id = id;
                    
                    let succs = match stmt {
                        Stmt::Return(_) => vec![exit_id],
                        Stmt::Branch(br_rc) => {
                            let bs: &BranchStmt = br_rc.as_ref();
                            match bs.token {
                                Token::BREAK => {
                                    if let Some(label_ident) = bs.label {
                                        let name = objs.idents[label_ident].name.clone();
                                        if let Some(&target) = label_map.get(&name) {
                                            // Labeled break
                                            vec![target]
                                        } else {
                                            // Unresolved labeled break
                                            pending_gotos.push((id, name));
                                            Vec::new()
                                        }
                                    } else if let Some(after) = loop_after {
                                        // Normal break
                                        vec![after]
                                    } else {
                                        // Break outside loop - goes to exit
                                        vec![exit_id]
                                    }
                                },
                                Token::CONTINUE => {
                                    if let Some(cond) = loop_cond {
                                        vec![cond]
                                    } else {
                                        // Continue outside loop - goes to exit
                                        vec![exit_id]
                                    }
                                },
                                Token::GOTO => {
                                    if let Some(label_ident) = bs.label {
                                        let name = objs.idents[label_ident].name.clone();
                                        if let Some(&target) = label_map.get(&name) {
                                            // Label already defined
                                            vec![target]
                                        } else {
                                            // Not resolved yet
                                            pending_gotos.push((id, name));
                                            Vec::new()
                                        }
                                    } else {
                                        Vec::new()
                                    }
                                }
                                _ => vec![],
                            }
                        }
                        _ => vec![],
                    };
                    
                    blocks.insert(id, BasicBlock { id, stmts: vec![stmt.clone()], succs });
                    if let Some(p) = prev {
                        if blocks.get(&p).unwrap().succs.is_empty() {
                            blocks.get_mut(&p).unwrap().succs = vec![id];
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
    _fset: &FileSet,
    objs: &AstObjects,
    file: &go_parser::ast::File,
) -> HashMap<String, ControlFlowGraph> {
    let mut out = HashMap::new();
    for decl in &file.decls {
        if let Decl::Func(fk) = *decl {
            let fdecl = &objs.fdecls[fk];
            let name = objs.idents[fdecl.name].name.clone();
            out.insert(name, ControlFlowGraph::build(fdecl, objs));
        }
    }
    out
}
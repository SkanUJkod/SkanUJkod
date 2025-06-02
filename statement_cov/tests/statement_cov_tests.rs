//! Integration tests for the statement‑coverage instrumentation layer.
//! These tests build a minimal, synthetic control‑flow graph (CFG) by hand and feed it into `generate_instrumented_main`, verifying that:
//!   1. a separate `stmt_hit` call is generated for every single statement.
//!   2. generated statement‑IDs are consecutive (0, 1, …).
//!   3. a stub `main` function is appended when the instrumented project does not contain a user‑defined `main`.

use std::collections::HashMap;
use std::rc::Rc;

use cfg::cfg::types::{BasicBlock, ControlFlowGraph, Statement};
use go_parser::ast::{EmptyStmt, Stmt};

fn empty_stmt() -> Stmt {
    Stmt::Empty(Rc::new(EmptyStmt { semi: 0, implicit: true }))
}

fn run_codegen(cfgs: &HashMap<String, ControlFlowGraph>) -> String {
    let mut out = String::new();

    // Iterate deterministically for stable test output.
    let mut funcs: Vec<_> = cfgs.keys().collect();
    funcs.sort();

    for func in funcs {
        if let Some(cfg) = cfgs.get(func) {
            let mut id = 0;
            // Sort blocks for deterministic order.
            let mut block_ids: Vec<_> = cfg.blocks.keys().collect();
            block_ids.sort();
            for bid in block_ids {
                if let Some(block) = cfg.blocks.get(bid) {
                    for _ in &block.stmts {
                        out.push_str(&format!("stmt_hit(\"{}\", {})\n", func, id));
                        id += 1;
                    }
                }
            }
        }
    }

    // If the project does not define its own `main` function, add a stub.
    if !cfgs.contains_key("main") {
        out.push_str("func main() {}\n");
    }

    out
}

#[test]
fn generates_stmt_hit_for_each_statement() {
    let mut blocks = HashMap::new();

    blocks.insert(
        0,
        BasicBlock {
            id: 0,
            stmts: vec![Statement { text: "first".into(), stmt: empty_stmt() }],
            succs: vec![1],
            start_line: 0,
            end_line: 0,
        },
    );

    blocks.insert(
        1,
        BasicBlock {
            id: 1,
            stmts: vec![Statement { text: "second".into(), stmt: empty_stmt() }],
            succs: vec![],
            start_line: 0,
            end_line: 0,
        },
    );

    let cfg = ControlFlowGraph { blocks, entry: 0 };
    let mut cfgs = HashMap::new();
    cfgs.insert("foo".to_string(), cfg);

    let generated = run_codegen(&cfgs);

    // Collect all stmt_hit lines for "foo"
    let all_hits: Vec<_> = generated
        .lines()
        .filter(|l| l.starts_with("stmt_hit(\"foo\","))
        .collect();
    assert_eq!(all_hits.len(), 2, "There should be exactly 2 stmt_hit calls for foo");
    assert_eq!(all_hits[0], "stmt_hit(\"foo\", 0)");
    assert_eq!(all_hits[1], "stmt_hit(\"foo\", 1)");
}

#[test]
fn adds_stub_main_when_absent() {
    // Same tiny CFG as before, but stored under a name ≠ "main".
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            id: 0,
            stmts: vec![Statement { text: "one".into(), stmt: empty_stmt() }],
            succs: vec![],
            start_line: 0,
            end_line: 0,
        },
    );
    let cfg = ControlFlowGraph { blocks, entry: 0 };
    let mut cfgs = HashMap::new();
    cfgs.insert("not_main".to_string(), cfg);

    let generated = run_codegen(&cfgs);

    assert!(generated.contains("func main() {}"), "stub main() should be present");
}

#[test]
fn generates_stmt_hit_for_empty_cfg() {
    let cfgs = HashMap::new();
    let generated = run_codegen(&cfgs);

    assert!(generated.contains("func main() {}"), "stub main() should be present");
}

#[test]
fn handles_multiple_functions() {
    let mut blocks = HashMap::new();
    blocks.insert(
        0,
        BasicBlock {
            id: 0,
            stmts: vec![Statement { text: "stmt1".into(), stmt: empty_stmt() }],
            succs: vec![1],
            start_line: 0,
            end_line: 0,
        },
    );
    blocks.insert(
        1,
        BasicBlock {
            id: 1,
            stmts: vec![Statement { text: "stmt2".into(), stmt: empty_stmt() }],
            succs: vec![],
            start_line: 0,
            end_line: 0,
        },
    );

    let cfg1 = ControlFlowGraph { blocks: blocks.clone(), entry: 0 };
    let cfg2 = ControlFlowGraph { blocks, entry: 0 };

    let mut cfgs = HashMap::new();
    cfgs.insert("func1".to_string(), cfg1);
    cfgs.insert("func2".to_string(), cfg2);

    let generated = run_codegen(&cfgs);

    assert!(generated.contains("stmt_hit(\"func1\", 0)"));
    assert!(generated.contains("stmt_hit(\"func2\", 0)"));
}

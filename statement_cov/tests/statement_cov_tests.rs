//! Integration tests for the statement‑coverage instrumentation layer.
//! These tests build a minimal, synthetic control‑flow graph (CFG) by hand and feed it into `generate_instrumented_main`, verifying that:
//!   1. a separate `stmt_hit` call is generated for every single statement.
//!   2. generated statement‑IDs are consecutive (0, 1, …).
//!   3. a stub `main` function is appended when the instrumented project does not contain a user‑defined `main`.

use std::collections::HashMap;
use std::rc::Rc;

use cfg::cfg::types::{BasicBlock, ControlFlowGraph, Statement};
use statement_cov::instrumentation::generate_instrumented_main;
use go_parser::ast::{EmptyStmt, Stmt};

/// Build a single empty `Stmt` helper (avoids bringing the entire parser in).
fn empty_stmt() -> Stmt {
    Stmt::Empty(Rc::new(EmptyStmt { semi: 0, implicit: true }))
}

#[test]
fn generates_stmt_hit_for_each_statement() {
    //  CFG layout:
    //  ┌── entry(0) ──► 1 ──► exit
    //  Block 0: one stmt
    //  Block 1: one stmt
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

    let generated = generate_instrumented_main(&cfgs)
        .expect("code generation must succeed");

    // Expect two distinct calls with IDs 0 and 1.
    assert!(generated.contains("stmt_hit(\"foo\", 0)"));
    assert!(generated.contains("stmt_hit(\"foo\", 1)"));

    // Ensure no accidental duplicate of ID 0.
    let id0_count = generated.matches("stmt_hit(\"foo\", 0)").count();
    assert_eq!(id0_count, 1, "ID 0 should appear exactly once");
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

    let generated = generate_instrumented_main(&cfgs)
        .expect("code generation must succeed");

    assert!(generated.contains("func main() {}"), "stub main() should be present");
}

#[test]
fn generates_stmt_hit_for_empty_cfg() {
    let cfgs = HashMap::new();
    let generated = generate_instrumented_main(&cfgs)
        .expect("code generation must succeed");

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

    let generated = generate_instrumented_main(&cfgs)
        .expect("code generation must succeed");

    println!("Generated code:\n{}", generated); // Debugging statement

    assert!(generated.contains("stmt_hit(\"func1\", 0)"));
    assert!(generated.contains("stmt_hit(\"func2\", 0)"));
}

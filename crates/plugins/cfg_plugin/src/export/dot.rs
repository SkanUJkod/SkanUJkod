use crate::cfg::ControlFlowGraph;
use go_parser::ast::Stmt as GoStmt;

#[allow(dead_code)]
fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn to_dot(cfg: &ControlFlowGraph) -> String {
    let mut out = String::new();
    
    for (id, block) in &cfg.blocks {
        let label_content = if block.stmts.is_empty() {
            if *id == cfg.entry {
                "entry".to_string()
            } else {
                "exit".to_string()
            }
        } else {
            match &block.stmts[0].stmt {
                GoStmt::If(_) => "If".to_string(),
                GoStmt::For(_) => "For".to_string(),
                GoStmt::Range(_) => "Range".to_string(),
                GoStmt::Switch(_) => "Switch".to_string(),
                GoStmt::Select(_) => "Select".to_string(),
                GoStmt::Return(_) => "Return".to_string(),
                GoStmt::Decl(_) => "Decl".to_string(),
                GoStmt::Assign(_) => "Assign".to_string(),
                GoStmt::Go(_) => "Go".to_string(),
                GoStmt::Defer(_) => "Defer".to_string(),
                GoStmt::Branch(_) => "Branch".to_string(),
                GoStmt::Empty(_) => "Empty".to_string(),
                GoStmt::Labeled(_) => "Labeled".to_string(),
                GoStmt::Send(_) => "Send".to_string(),
                GoStmt::IncDec(_) => "IncDec".to_string(),
                GoStmt::Comm(_) => "Comm".to_string(),
                GoStmt::Block(_) => "Block".to_string(),
                GoStmt::Expr(_) => "Expr".to_string(),
                _ => "Stmt".to_string(),
            }
        };
        let safe_label = label_content.replace('\\', "\\\\").replace('\"', "\\\"");
        out.push_str(&format!("    n{} [label=\"{}\"];\n", id, safe_label));
        for succ in &block.succs {
            out.push_str(&format!("    n{} -> n{};\n", id, succ));
        }
    }
    
    out
}

use crate::cfg::ControlFlowGraph;

pub fn to_dot(cfg: &ControlFlowGraph, name: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("digraph {} {{\n", name));
    for (id, block) in &cfg.blocks {
        let label: String = if block.stmts.is_empty() {
            if *id == cfg.entry {
                "entry".to_string()
            } else {
                "exit".to_string()
            }
        } else {
            format!("{:?}", block.stmts[0])
                .split_once('(')
                .map(|(s, _)| s.to_string())
                .unwrap_or_else(|| "stmt".to_string())
        };
        out.push_str(&format!("  n{} [label=\"{}\"];\n", id, label));
        for succ in &block.succs {
            out.push_str(&format!("  n{} -> n{};\n", id, succ));
        }
    }
    out.push_str("}\n");
    out
}
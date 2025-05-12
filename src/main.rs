mod stmt_cov;

use std::env;
use anyhow::Result;
use std::path::PathBuf;
use stmt_cov::run_statement_coverage;

fn main() -> Result<()> {
    let project = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let coverage = run_statement_coverage(&project)?;
    println!("Statement coverage: {:.2}%", coverage);

    Ok(())
}
pub mod core;
pub mod metrics;
pub mod types;
pub mod utils;

use std::collections::HashMap;

use crate::core::repo::RepoWrapper;
use crate::core::runner::run_selected_metrics;

pub fn run_metrics(
    repo_path: &str,
    selected: &[&str],
    all_params: &HashMap<String, HashMap<String, String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = RepoWrapper::new(repo_path)?;

    run_selected_metrics(&repo, selected, all_params);

    Ok(())
}

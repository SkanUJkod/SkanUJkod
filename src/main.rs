mod metrics;
mod repo;
mod runner;

use repo::RepoWrapper;
use runner::run_selected_metrics;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo = RepoWrapper::new(".").unwrap();
    let selected = vec!["contributors_in_timeframe"];

    let mut all_params: HashMap<String, HashMap<String, String>> = HashMap::new();

    let end_date = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let start_date = end_date - (30 * 24 * 60 * 60);

    all_params.insert(
        "contributors_in_timeframe".to_string(),
        HashMap::from([
            ("start_date".to_string(), start_date.to_string()),
            ("end_date".to_string(), end_date.to_string()),
        ]),
    );

    run_selected_metrics(&repo, &selected, &all_params);

    Ok(())
}

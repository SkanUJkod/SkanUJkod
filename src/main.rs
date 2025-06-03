mod git_metrics;

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::git_metrics::core::repo::RepoWrapper;
use crate::git_metrics::core::runner::run_selected_metrics;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo = RepoWrapper::new(".").unwrap();
    let selected = vec![
        "commits_by_author_in_repo",
        "contributors_in_timeframe",
        "percentage_of_total_commits",
        "first_last_commit",
        "lines_added_removed",
    ];

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

    all_params.insert(
        "lines_added_removed".to_string(),
        HashMap::from([("author".to_string(), "Jakub Magiera".to_string())]),
    );

    run_selected_metrics(&repo, &selected, &all_params);

    Ok(())
}

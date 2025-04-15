use crate::metrics::all_metrics;
use crate::repo::RepoWrapper;
use std::collections::HashMap;

pub fn run_selected_metrics(
    repo: &RepoWrapper,
    selected: Vec<&str>,
    all_params: &HashMap<String, HashMap<String, String>>,
) {
    let metrics = all_metrics();

    for metric in metrics {
        if selected.contains(&metric.name()) {
            let default_params = HashMap::new();
            let params = all_params
                .get(metric.name())
                .unwrap_or(&default_params);
            metric.run(repo, params);
        }
    }
}

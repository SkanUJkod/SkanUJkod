use crate::metrics::{all_metrics, metrics_trait::Metric, result_type::MetricResultType, utils::print_results};
use crate::repo::RepoWrapper;
use std::collections::HashMap;

pub fn run_selected_metrics(
    repo_wrapper: &RepoWrapper,
    selected: Vec<&str>,
    all_params: &HashMap<String, HashMap<String, String>>,
) {
    let metrics = all_metrics();
    let selected_metrics: Vec<&dyn Metric> = metrics
        .iter()
        .filter(|metric| selected.contains(&metric.name()))
        .map(|metric| metric.as_ref())
        .collect();

    let repo = &repo_wrapper.repo;
    let mut commit_id = repo.head_commit().unwrap().id();
    let mut results: HashMap<&str, MetricResultType> = HashMap::new();

    for metric in &selected_metrics {
        let metric_name = metric.name();
        results.insert(metric_name, metric.default_results());
    }

    while let Ok(commit) = repo.find_commit(commit_id) {
        for metric in &selected_metrics {
            let default_params = HashMap::new();
            let params = all_params
                .get(metric.name())
                .unwrap_or(&default_params);
            
            if let Some(result) = results.get_mut(metric.name()) {
                metric.run(&commit, params, result);
            }
        }

        if let Some(parent_id) = commit.parent_ids().next() {
            commit_id = parent_id;
        } else {
            break;
        }
    }

    print_results(&selected_metrics, &results);
}

use gix::Repository;
use crate::metrics::{
    all_metrics, metrics_trait::Metric, result_type::MetricResultType, utils::print_results,
};
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
    let mut results = init_empty_results(&selected_metrics);

    let all_commits_info = get_all_commits(repo);

    for commit_info in all_commits_info {
        let commit_info = commit_info.unwrap();
        let commit = repo.find_commit(commit_info.id()).unwrap();
        for metric in &selected_metrics {
            let default_params = HashMap::new();
            let params = all_params.get(metric.name()).unwrap_or(&default_params);
            if let Some(result) = results.get_mut(metric.name()) {
                metric.run(&commit, &params, result);
            }
        }
    }

    for metric in &selected_metrics {
        if let Some(result) = results.get_mut(metric.name()) {
            metric.calculate(result);
        }
    }

    print_results(&selected_metrics, &results);
}

fn init_empty_results<'a>(metrics: &'a Vec<&dyn Metric>) -> HashMap<&'a str, MetricResultType> {
    metrics
        .iter()
        .map(|metric| (metric.name(), metric.default_results()))
        .collect()
}

fn get_all_commits(repo: &Repository) -> gix::revision::Walk {
    let head_commit = repo.head_commit().unwrap();
    let commit = repo.find_commit(head_commit.id()).unwrap();
    let commits = commit.ancestors().all().unwrap();

    commits
}

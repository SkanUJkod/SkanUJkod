use crate::metrics::{
    all_metrics, metrics_trait::Metric, result_type::MetricResultType, utils::print_results,
};
use crate::repo::RepoWrapper;
use gix::Repository;
use std::collections::HashMap;

pub fn run_selected_metrics(
    repo_wrapper: &RepoWrapper,
    selected: &[&str],
    all_params: &HashMap<String, HashMap<String, String>>,
) {
    let metrics = all_metrics();
    let selected_with_deps = resolve_metric_dependencies(&metrics, selected);
    let selected_metrics_with_deps: Vec<&dyn Metric> = metrics
        .iter()
        .filter(|metric| selected_with_deps.contains(&metric.name()))
        .map(AsRef::as_ref)
        .collect();

    let repo = &repo_wrapper.repo;
    let mut results = init_empty_results(&selected_metrics_with_deps);

    let mut all_commits_info: Vec<_> = get_all_commits(repo).collect();
    all_commits_info.reverse();

    for i in 0..all_commits_info.len() {
        let commit_info = all_commits_info[i].as_ref().unwrap();
        let commit = repo.find_commit(commit_info.id()).unwrap();

        let child_commit = if i + 1 < all_commits_info.len() {
            let child_info = all_commits_info[i + 1].as_ref().unwrap();
            Some(repo.find_commit(child_info.id()).unwrap())
        } else {
            None
        };

        for metric in &selected_metrics_with_deps {
            let default_params = HashMap::new();
            let params = all_params.get(metric.name()).unwrap_or(&default_params);
            if let Some(result) = results.get_mut(metric.name()) {
                metric.run(&commit, child_commit.as_ref(), params, result);
            }
        }
    }

    for metric in &selected_metrics_with_deps {
        if let Some(dependency_name) = metric.dependencies() {
            if let Some(dependency_result) = results.get(dependency_name).cloned() {
                results.insert(metric.name(), dependency_result);
            }
        }
        if let Some(result) = results.get_mut(metric.name()) {
            metric.calculate(result);
        }
    }

    let selected_metrics_without_deps: Vec<&dyn Metric> = metrics
        .iter()
        .filter(|metric| selected.contains(&metric.name()))
        .map(AsRef::as_ref)
        .collect();

    print_results(&selected_metrics_without_deps, &results);
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

fn resolve_metric_dependencies<'a>(
    metrics: &'a Vec<Box<dyn Metric>>,
    selected: &[&'a str],
) -> Vec<&'a str> {
    let mut stack = selected.to_vec();
    let mut resolved = Vec::new();
    while let Some(metric_name) = stack.pop() {
        if !resolved.contains(&metric_name) {
            resolved.push(metric_name);
        }

        if let Some(metric) = metrics.iter().find(|m| m.name() == metric_name) {
            if let Some(dep) = metric.dependencies() {
                if !resolved.contains(&dep) && !stack.contains(&dep) {
                    stack.push(dep);
                    resolved.push(dep);
                }
            }
        }
    }
    resolved
}

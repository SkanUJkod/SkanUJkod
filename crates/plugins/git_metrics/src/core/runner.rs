use crate::core::repo::RepoWrapper;
use crate::metrics::all_metrics;
use crate::types::metrics_trait::Metric;
use crate::types::result_type::MetricResultType;
use crate::utils::printer::print_results;
use gix::Repository;
use gix::revision::Walk;
use std::collections::HashMap;

pub fn run_selected_metrics(
    repo_wrapper: &RepoWrapper,
    selected: &[&str],
    all_params: &HashMap<String, HashMap<String, String>>,
) -> HashMap<String, MetricResultType> {
    let metrics = all_metrics();
    let selected_with_deps = resolve_metric_dependencies(&metrics, selected);

    let selected_metrics_with_deps: Vec<&dyn Metric> = metrics
        .iter()
        .filter(|metric| selected_with_deps.contains(&metric.name()))
        .map(AsRef::as_ref)
        .collect();

    let repo = &repo_wrapper.repo;
    let mut results = init_empty_results(&selected_metrics_with_deps);

    compute_from_commits(repo, &selected_metrics_with_deps, all_params, &mut results);
    finalize_metrics(&selected_metrics_with_deps, &mut results);

    let selected_metrics_without_deps: Vec<&dyn Metric> = metrics
        .iter()
        .filter(|metric| selected.contains(&metric.name()))
        .map(AsRef::as_ref)
        .collect();

    print_results(&selected_metrics_without_deps, &results);

    results
}

fn init_empty_results<'a>(metrics: &'a Vec<&dyn Metric>) -> HashMap<String, MetricResultType> {
    metrics
        .iter()
        .map(|metric| (metric.name().to_string(), metric.default_result()))
        .collect()
}

fn get_all_commits(repo: &Repository) -> Walk {
    let head_commit = repo.head_commit().unwrap();
    let commit = repo.find_commit(head_commit.id()).unwrap();
    commit.ancestors().all().unwrap()
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
            if let Some(dep) = metric.dependency() {
                if !resolved.contains(&dep) && !stack.contains(&dep) {
                    stack.push(dep);
                    resolved.push(dep);
                }
            }
        }
    }
    resolved
}

fn compute_from_commits(
    repo: &Repository,
    selected_metrics_with_deps: &[&dyn Metric],
    all_params: &HashMap<String, HashMap<String, String>>,
    results: &mut HashMap<String, MetricResultType>,
) {
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

        for metric in selected_metrics_with_deps {
            let default_params = HashMap::new();
            let params = all_params.get(metric.name()).unwrap_or(&default_params);
            if let Some(result) = results.get_mut(metric.name()) {
                metric.compute(&commit, child_commit.as_ref(), params, result);
            }
        }
    }
}

fn finalize_metrics(
    selected_metrics_with_deps: &[&dyn Metric],
    results: &mut HashMap<String, MetricResultType>,
) {
    for metric in selected_metrics_with_deps {
        if let Some(dependency_name) = metric.dependency() {
            if let Some(dependency_result) = results.get(dependency_name).cloned() {
                results.insert(metric.name().to_string(), dependency_result);
            }
        }
        if let Some(result) = results.get_mut(metric.name()) {
            metric.finalize(result);
        }
    }
}

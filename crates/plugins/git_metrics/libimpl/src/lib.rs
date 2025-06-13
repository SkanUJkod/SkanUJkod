pub mod core;
pub mod metrics;
pub mod types;
pub mod utils;

use std::collections::HashMap;

use crate::core::repo::RepoWrapper;
use crate::core::runner::run_selected_metrics;
use crate::metrics::commits::CommitsByAuthorInRepo;
use crate::types::metrics_trait::Metric;

pub fn run_metrics(
    repo_path: &str,
    selected: &[&str],
    all_params: &HashMap<String, HashMap<String, String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = RepoWrapper::new(repo_path)?;

    run_selected_metrics(&repo, selected, all_params);

    Ok(())
}

fn all_commits<'a>(repo: &'a gix::Repository) -> Vec<gix::Commit<'a>> {
    let mut commits = repo
        .head_commit()
        .expect("Repository does not contain a HEAD commit (empty?)")
        .ancestors()
        .all()
        .expect("Something went wrong when trying to query all ancestors of HEAD commit")
        .flatten()
        .map(|info| {
            repo.find_commit(info.id())
                .expect("Internal error: gix traversal returned non-commit id")
        })
        .collect::<Vec<_>>();
    commits.reverse();
    commits
}

fn windows2_opt<T>(v: &Vec<T>) -> impl Iterator<Item = (&T, Option<&T>)> {
    let iter1 = v.as_slice().windows(2).map(|sl| (&sl[0], Some(&sl[1])));
    let iter2 = [(v.last().unwrap(), None)].into_iter();
    iter1.chain(iter2)
}

pub struct ReadRepoDeps {}

pub struct ReadRepoParams {
    pub dir: String,
}

#[derive(Debug)]
pub struct ReadRepoResult {
    result: gix::Repository,
}

impl std::fmt::Display for ReadRepoResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

pub fn read_repo(_deps: ReadRepoDeps, params: ReadRepoParams) -> ReadRepoResult {
    ReadRepoResult {
        result: RepoWrapper::new(params.dir.as_str())
            .expect("Failed to read repository")
            .repo,
    }
}

pub struct CommitsByAuthorDeps<'a> {
    pub read_repo: &'a ReadRepoResult,
}

pub struct CommitsByAuthorParams {}

#[derive(Debug)]
pub struct CommitsByAuthorResult {
    result: std::collections::HashMap<String, u32>,
}

impl std::fmt::Display for CommitsByAuthorResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

pub fn commits_by_author(
    deps: CommitsByAuthorDeps,
    _params: CommitsByAuthorParams,
) -> CommitsByAuthorResult {
    let metric = CommitsByAuthorInRepo {};
    let mut result = metric.default_result();
    // let metric_params = {
    //     let mut ret: HashMap<String, String> = HashMap::new();
    //     ret.insert("start_date".into(), params.start_date.to_string());
    //     ret.insert("end_date".into(), params.end_date.to_string());
    //     ret
    // };

    windows2_opt(&all_commits(&deps.read_repo.result)).for_each(|(commit, child_commit)| {
        metric.compute(commit, child_commit, &HashMap::new(), &mut result)
    });

    match result {
        types::result_type::MetricResultType::CountMap(hash_map) => {
            CommitsByAuthorResult { result: hash_map }
        }
        _ => panic!(),
    }
}

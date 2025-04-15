use std::collections::{HashMap, HashSet};
use crate::metrics::Metric;
use crate::repo::RepoWrapper;

pub struct CommitsByAuthorInRepo;
pub struct ContributorsInTimeframe;

impl Metric for CommitsByAuthorInRepo {
    fn name(&self) -> &'static str {
        "commits_by_author_in_repo"
    }
    
    fn run(&self, repo_wrapper: &RepoWrapper, _params: &HashMap<String, String>) {
        let repo = &repo_wrapper.repo;
        let mut authors_commits: HashMap<String, i32> = HashMap::new();
        let mut commit_id = repo.head_commit().unwrap().id();

        while let Ok(commit) = repo.find_commit(commit_id) {
            let author_name = commit.author().unwrap().name.to_string();
            *authors_commits.entry(author_name).or_insert(0) += 1;

            if let Some(parent_id) = commit.parent_ids().next() {
                commit_id = parent_id;
            } else {
                break;
            }
        }

        for (author, count) in &authors_commits {
            println!("{}: {}", author, count);
        }

        //Ok(authors_commits)
    }
}

impl Metric for ContributorsInTimeframe {
    fn name(&self) -> &'static str {
        "contributors_in_timeframe"
    }
    
    fn run(&self, repo_wrapper: &RepoWrapper, params: &HashMap<String, String>) {
        let start_date = params.get("start_date")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(10);

        let end_date = params.get("end_date")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(10);

        let repo = &repo_wrapper.repo;
        let mut contributors: HashSet<String> = HashSet::new();
        let mut commit_id = repo.head_commit().unwrap().id();

        while let Ok(commit) = repo.find_commit(commit_id) {
            let commit_time = commit.time().unwrap().seconds;
            if commit_time >= start_date && commit_time <= end_date {
                let author_name = commit.author().unwrap().name.to_string();
                contributors.insert(author_name);
            }

            if let Some(parent_id) = commit.parent_ids().next() {
                commit_id = parent_id;
            } else {
                break;
            }
        }

        for author in &contributors {
            println!("{}", author);
        }

        //Ok(contributors)
    }
}

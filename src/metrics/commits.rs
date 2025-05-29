use super::result_type::MetricResultType;
use crate::metrics::{Metric, utils::parse_param_i64};
use chrono::DateTime;
use gix::Commit;
use std::collections::{HashMap, HashSet};

pub struct CommitsByAuthorInRepo;
pub struct ContributorsInTimeframe;
pub struct PercentageOfTotalCommits;
pub struct FirstLastCommit;

impl Metric for CommitsByAuthorInRepo {
    fn name(&self) -> &str {
        "commits_by_author_in_repo"
    }

    fn default_results(&self) -> super::result_type::MetricResultType {
        super::result_type::MetricResultType::Map(HashMap::new())
    }

    fn run(
        &self,
        commit: &Commit,
        _child_commit: &Option<Commit>,
        _params: &HashMap<String, String>,
        result: &mut MetricResultType,
    ) {
        match result {
            MetricResultType::Map(authors_commits) => {
                if let Ok(author) = commit.author() {
                    *authors_commits.entry(author.name.to_string()).or_insert(0) += 1;
                }
            }
            _ => {
                // Handle other types of MetricResultType
            }
        }
    }
}

impl Metric for ContributorsInTimeframe {
    fn name(&self) -> &str {
        "contributors_in_timeframe"
    }

    fn default_results(&self) -> super::result_type::MetricResultType {
        super::result_type::MetricResultType::Set(HashSet::new())
    }

    fn run(
        &self,
        commit: &Commit,
        _child_commit: &Option<Commit>,
        params: &HashMap<String, String>,
        result: &mut MetricResultType,
    ) {
        let start_date = parse_param_i64(params, "start_date", 10);
        let end_date = parse_param_i64(params, "end_date", 10);

        match result {
            MetricResultType::Set(contributors) => {
                let commit_time = commit.time().unwrap().seconds;
                if commit_time >= start_date && commit_time <= end_date {
                    let author_name = commit.author().unwrap().name.to_string();
                    contributors.insert(author_name);
                }
            }
            _ => {
                // Handle other types of MetricResultType
            }
        }
    }
}

impl Metric for PercentageOfTotalCommits {
    fn name(&self) -> &str {
        "percentage_of_total_commits"
    }

    fn default_results(&self) -> super::result_type::MetricResultType {
        super::result_type::MetricResultType::Map(HashMap::new())
    }

    fn dependencies(&self) -> Option<&str> {
        Some("commits_by_author_in_repo")
    }

    fn calculate(&self, result: &mut MetricResultType) {
        match result {
            MetricResultType::Map(authors_commits) => {
                let total: i32 = authors_commits.values().sum();
                if total > 0 {
                    for value in authors_commits.values_mut() {
                        *value = (*value as f64 / total as f64 * 100.0).round() as i32;
                    }
                }
            }
            _ => {
                // Handle other types of MetricResultType
            }
        }
    }
}

impl Metric for FirstLastCommit {
    fn name(&self) -> &str {
        "first_last_commit"
    }

    fn default_results(&self) -> super::result_type::MetricResultType {
        super::result_type::MetricResultType::DatePair(HashMap::new())
    }

    fn run(
        &self,
        commit: &Commit,
        _child_commit: &Option<Commit>,
        _params: &HashMap<String, String>,
        result: &mut MetricResultType,
    ) {
        match result {
            MetricResultType::DatePair(first_last_commit) => {
                let author_name = commit.author().unwrap().name.to_string();
                let commit_time = commit.time().unwrap().seconds;
                let datetime_utc = DateTime::from_timestamp(commit_time, 0);

                first_last_commit
                    .entry(author_name.clone())
                    .and_modify(|e| {
                        e.1 = datetime_utc.unwrap();
                    })
                    .or_insert((datetime_utc.unwrap(), datetime_utc.unwrap()));
            }
            _ => {
                // Handle other types of MetricResultType
            }
        }
    }
}

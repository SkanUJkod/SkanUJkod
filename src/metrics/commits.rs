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
    fn name(&self) -> &'static str {
        "commits_by_author_in_repo"
    }

    fn default_results(&self) -> super::result_type::MetricResultType {
        super::result_type::MetricResultType::CountMap(HashMap::new())
    }

    fn run(
        &self,
        commit: &Commit,
        _child_commit: Option<&Commit>,
        _params: &HashMap<String, String>,
        result: &mut MetricResultType,
    ) {
        if let MetricResultType::CountMap(authors_commits) = result {
            if let Ok(author) = commit.author() {
                *authors_commits.entry(author.name.to_string()).or_insert(0) += 1;
            }
        }
    }
}

impl Metric for ContributorsInTimeframe {
    fn name(&self) -> &'static str {
        "contributors_in_timeframe"
    }

    fn default_results(&self) -> super::result_type::MetricResultType {
        super::result_type::MetricResultType::UniqueValues(HashSet::new())
    }

    fn run(
        &self,
        commit: &Commit,
        _child_commit: Option<&Commit>,
        params: &HashMap<String, String>,
        result: &mut MetricResultType,
    ) {
        let start_date = parse_param_i64(params, "start_date", 10);
        let end_date = parse_param_i64(params, "end_date", 10);

        if let MetricResultType::UniqueValues(contributors) = result {
            let commit_time = commit.time().unwrap().seconds;
            if commit_time >= start_date && commit_time <= end_date {
                let author_name = commit.author().unwrap().name.to_string();
                contributors.insert(author_name);
            }
        }
    }
}

impl Metric for PercentageOfTotalCommits {
    fn name(&self) -> &'static str {
        "percentage_of_total_commits"
    }

    fn default_results(&self) -> super::result_type::MetricResultType {
        super::result_type::MetricResultType::CountMap(HashMap::new())
    }

    fn dependencies(&self) -> Option<&str> {
        Some("commits_by_author_in_repo")
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn calculate(&self, result: &mut MetricResultType) {
        if let MetricResultType::CountMap(authors_commits) = result {
            let total: u32 = authors_commits.values().sum();
            if total > 0 {
                for value in authors_commits.values_mut() {
                    *value = (f64::from(*value) / f64::from(total) * 100.0).round() as u32;
                }
            }
        }
    }
}

impl Metric for FirstLastCommit {
    fn name(&self) -> &'static str {
        "first_last_commit"
    }

    fn default_results(&self) -> super::result_type::MetricResultType {
        super::result_type::MetricResultType::TimeRange(HashMap::new())
    }

    fn run(
        &self,
        commit: &Commit,
        _child_commit: Option<&Commit>,
        _params: &HashMap<String, String>,
        result: &mut MetricResultType,
    ) {
        if let MetricResultType::TimeRange(first_last_commit) = result {
            let author_name = commit.author().unwrap().name.to_string();
            let commit_time = commit.time().unwrap().seconds;
            let datetime_utc = DateTime::from_timestamp(commit_time, 0);

            first_last_commit
                .entry(author_name)
                .and_modify(|e| {
                    e.1 = datetime_utc.unwrap();
                })
                .or_insert_with(|| (datetime_utc.unwrap(), datetime_utc.unwrap()));
        }
    }
}

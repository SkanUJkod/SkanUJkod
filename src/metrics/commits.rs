use super::result_type::MetricResultType;
use crate::metrics::{Metric, utils::parse_param};
use gix::Commit;
use std::collections::{HashMap, HashSet};

pub struct CommitsByAuthorInRepo;
pub struct ContributorsInTimeframe;

impl Metric for CommitsByAuthorInRepo {
    fn name(&self) -> &'static str {
        "commits_by_author_in_repo"
    }

    fn default_results(&self) -> super::result_type::MetricResultType {
        super::result_type::MetricResultType::Map(HashMap::new())
    }

    fn run(
        &self,
        commit: &Commit,
        _params: &HashMap<String, String>,
        result: &mut MetricResultType,
    ) {
        match result {
            MetricResultType::Map(authors_commits) => {
                let author_name = commit.author().unwrap().name.to_string();
                *authors_commits.entry(author_name).or_insert(0) += 1;
            }
            _ => {
                // Handle other types of MetricResultType
            }
        }
    }
}

impl Metric for ContributorsInTimeframe {
    fn name(&self) -> &'static str {
        "contributors_in_timeframe"
    }

    fn default_results(&self) -> super::result_type::MetricResultType {
        super::result_type::MetricResultType::Set(HashSet::new())
    }

    fn run(
        &self,
        commit: &Commit,
        params: &HashMap<String, String>,
        result: &mut MetricResultType,
    ) {
        let start_date = parse_param(params, "start_date", 10);
        let end_date = parse_param(params, "end_date", 10);

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

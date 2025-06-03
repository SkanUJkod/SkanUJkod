pub mod commits;
pub mod ownership;

use commits::{
    CommitsByAuthorInRepo, ContributorsInTimeframe, FirstLastCommit, PercentageOfTotalCommits,
};
use ownership::LinesAddedRemoved;

use crate::git_metrics::types::metrics_trait::Metric;

pub fn all_metrics() -> Vec<Box<dyn Metric>> {
    vec![
        Box::new(CommitsByAuthorInRepo),
        Box::new(ContributorsInTimeframe),
        Box::new(PercentageOfTotalCommits),
        Box::new(FirstLastCommit),
        Box::new(LinesAddedRemoved),
    ]
}

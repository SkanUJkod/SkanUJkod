pub mod commits;
pub mod metrics_trait;
pub mod ownership;
pub mod result_type;
pub mod utils;

use commits::{
    CommitsByAuthorInRepo, ContributorsInTimeframe, FirstLastCommit, PercentageOfTotalCommits,
};
use metrics_trait::Metric;
use ownership::LinesAddedRemoved;

pub fn all_metrics() -> Vec<Box<dyn Metric>> {
    vec![
        Box::new(CommitsByAuthorInRepo),
        Box::new(ContributorsInTimeframe),
        Box::new(PercentageOfTotalCommits),
        Box::new(FirstLastCommit),
        Box::new(LinesAddedRemoved),
    ]
}

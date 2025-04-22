pub mod commits;
pub mod metrics_trait;
pub mod result_type;
pub mod utils;

use commits::{CommitsByAuthorInRepo, ContributorsInTimeframe};
use metrics_trait::Metric;

pub fn all_metrics() -> Vec<Box<dyn Metric>> {
    vec![
        Box::new(CommitsByAuthorInRepo),
        Box::new(ContributorsInTimeframe),
    ]
}

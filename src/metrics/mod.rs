pub mod metrics_trait;
pub mod commits;

use metrics_trait::Metric;
use commits::{ CommitsByAuthorInRepo, ContributorsInTimeframe };

pub fn all_metrics() -> Vec<Box<dyn Metric>> {
    vec![
        Box::new(CommitsByAuthorInRepo),
        Box::new(ContributorsInTimeframe),
    ]
}

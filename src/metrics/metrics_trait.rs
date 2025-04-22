use std::collections::HashMap;
use gix::Commit;
use crate::metrics::result_type::MetricResultType;

pub trait Metric {
    fn name(&self) -> &'static str;
    fn default_results(&self) -> MetricResultType;
    fn run(&self, repo: &Commit, params: &HashMap<String, String>, result: &mut MetricResultType);
}

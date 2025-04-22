use crate::metrics::result_type::MetricResultType;
use gix::Commit;
use std::collections::HashMap;

pub trait Metric {
    fn name(&self) -> &'static str;
    fn default_results(&self) -> MetricResultType;
    fn run(&self, repo: &Commit, params: &HashMap<String, String>, result: &mut MetricResultType);
}

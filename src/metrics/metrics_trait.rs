use crate::metrics::result_type::MetricResultType;
use gix::Commit;
use std::collections::HashMap;

pub trait Metric {
    fn name(&self) -> &str;
    fn default_results(&self) -> MetricResultType;
    fn run(
        &self,
        commit: &Commit,
        _child_commit: &Option<Commit>,
        params: &HashMap<String, String>,
        result: &mut MetricResultType,
    );
    fn calculate(&self, _result: &mut MetricResultType) {}
}

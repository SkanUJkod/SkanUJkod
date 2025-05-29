use crate::metrics::result_type::MetricResultType;
use gix::Commit;
use std::collections::HashMap;

pub trait Metric {
    fn name(&self) -> &'static str;
    fn default_results(&self) -> MetricResultType;
    fn dependencies(&self) -> Option<&str> {
        None
    }
    fn run(
        &self,
        _commit: &Commit,
        _child_commit: Option<&Commit>,
        _params: &HashMap<String, String>,
        _result: &mut MetricResultType,
    ) {
    }
    fn calculate(&self, _result: &mut MetricResultType) {}
}

use gix::Commit;
use std::collections::HashMap;

use crate::git_metrics::types::result_type::MetricResultType;

pub trait Metric {
    fn name(&self) -> &'static str;
    fn default_result(&self) -> MetricResultType;
    fn dependency(&self) -> Option<&str> {
        None
    }
    fn compute(
        &self,
        _commit: &Commit,
        _child_commit: Option<&Commit>,
        _params: &HashMap<String, String>,
        _result: &mut MetricResultType,
    ) {
    }
    fn finalize(&self, _result: &mut MetricResultType) {}
}

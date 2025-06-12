use std::collections::HashMap;

use crate::types::{metrics_trait::Metric, result_type::MetricResultType};

pub fn print_results(metrics: &Vec<&dyn Metric>, all_results: &HashMap<String, MetricResultType>) {
    for metric in metrics {
        let name = metric.name();
        let result = all_results.get(name).unwrap();
        println!("--- {name} ---");
        println!("{result:#?}");
        println!("--- --- ---\n");
    }
}

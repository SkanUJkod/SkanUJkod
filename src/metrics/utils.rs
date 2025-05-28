use super::{metrics_trait::Metric, result_type::MetricResultType};
use std::collections::HashMap;

pub fn parse_param_i64(params: &HashMap<String, String>, key: &str, default: i64) -> i64 {
    params
        .get(key)
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(default)
}

pub fn parse_param_string<'a>(
    params: &'a HashMap<String, String>,
    key: &str,
    default: &'a String,
) -> &'a String {
    params.get(key).unwrap_or(default)
}

pub fn print_results(metrics: &Vec<&dyn Metric>, all_results: &HashMap<&str, MetricResultType>) {
    for metric in metrics {
        let name = metric.name();
        let result = all_results.get(name).unwrap();
        println!("--- {} ---", name);
        println!("{:#?}", result);
        println!("--- --- ---\n");
    }
}

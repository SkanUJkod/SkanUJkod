use super::{metrics_trait::Metric, result_type::MetricResultType};
use std::collections::HashMap;

pub fn parse_param(params: &HashMap<String, String>, key: &str, default: i64) -> i64 {
    params
        .get(key)
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(default)
}

pub fn print_results(metrics: &Vec<&dyn Metric>, all_results: &HashMap<&str, MetricResultType>) {
    for metric in metrics {
        let name = metric.name();
        let result = all_results.get(name).unwrap();
        println!("--- {} ---", name);
        match result {
            MetricResultType::Map(map) => {
                for (key, value) in map {
                    println!("{}: {}", key, value);
                }
            }
            MetricResultType::Set(set) => {
                for value in set {
                    println!("{}", value);
                }
            }
            MetricResultType::DatePair(map) => {
                for (key, value) in map {
                    println!("{}: {} --- {}", key, value.0, value.1);
                }
            }
        }
        println!("--- --- ---\n");
    }
}

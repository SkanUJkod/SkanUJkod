use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub enum MetricResultType {
    Map(HashMap<String, i32>),
    DatePair(HashMap<String, (DateTime<Utc>, DateTime<Utc>)>),
    Set(HashSet<String>),
}

impl Clone for MetricResultType {
    fn clone(&self) -> Self {
        match self {
            MetricResultType::Map(map) => MetricResultType::Map(map.clone()),
            MetricResultType::DatePair(date_pair) => MetricResultType::DatePair(date_pair.clone()),
            MetricResultType::Set(set) => MetricResultType::Set(set.clone()),
        }
    }
}

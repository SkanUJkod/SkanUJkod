use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq)]
pub enum MetricResultType {
    CountMap(HashMap<String, u32>),
    TimeRange(HashMap<String, (DateTime<Utc>, DateTime<Utc>)>),
    UniqueValues(HashSet<String>),
}

impl Clone for MetricResultType {
    fn clone(&self) -> Self {
        match self {
            Self::CountMap(map) => Self::CountMap(map.clone()),
            Self::TimeRange(date_pair) => Self::TimeRange(date_pair.clone()),
            Self::UniqueValues(set) => Self::UniqueValues(set.clone()),
        }
    }
}

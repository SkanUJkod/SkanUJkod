use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub enum MetricResultType {
    Map(HashMap<String, u32>),
    DatePair(HashMap<String, (DateTime<Utc>, DateTime<Utc>)>),
    Set(HashSet<String>),
}

impl Clone for MetricResultType {
    fn clone(&self) -> Self {
        match self {
            Self::Map(map) => Self::Map(map.clone()),
            Self::DatePair(date_pair) => Self::DatePair(date_pair.clone()),
            Self::Set(set) => Self::Set(set.clone()),
        }
    }
}

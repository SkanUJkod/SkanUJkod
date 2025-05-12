use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub enum MetricResultType {
    Map(HashMap<String, i32>),
    DatePair(HashMap<String, (DateTime<Utc>, DateTime<Utc>)>),
    Set(HashSet<String>),
}

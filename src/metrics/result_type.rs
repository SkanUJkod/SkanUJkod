use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

pub enum MetricResultType {
    Map(HashMap<String, i32>),
    DatePair(HashMap<String, (DateTime<Utc>, DateTime<Utc>)>),
    Set(HashSet<String>),
}

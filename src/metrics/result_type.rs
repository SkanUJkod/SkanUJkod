use std::collections::{HashMap, HashSet};

pub enum MetricResultType {
    Map(HashMap<String, i32>),
    Set(HashSet<String>),
}

use crate::repo::RepoWrapper;
use std::collections::HashMap;

pub trait Metric {
    fn name(&self) -> &'static str;
    fn run(&self, repo: &RepoWrapper, params: &HashMap<String, String>);
}

use super::result_type::MetricResultType;
use crate::metrics::Metric;
use gix::Commit;
use imara_diff::Algorithm::Histogram;
use imara_diff::intern::InternedInput;
use std::{collections::HashMap, str};

pub struct LinesAddedRemoved;

impl Metric for LinesAddedRemoved {
    fn name(&self) -> &'static str {
        "lines_added_removed"
    }

    fn default_results(&self) -> super::result_type::MetricResultType {
        super::result_type::MetricResultType::Map(HashMap::new())
    }

    fn run(
        &self,
        commit: &Commit,
        _child_commit: &Option<gix::Commit>,
        _params: &HashMap<String, String>,
        result: &mut MetricResultType,
    ) {
        match result {
            MetricResultType::Map(lines_map) => {
                let _child_commit = match _child_commit {
                    Some(c) => c,
                    None => return,
                };

                let current_files = files_in_commit(commit);
                let child_files = files_in_commit(_child_commit);

                let mut total_added = 0;
                let mut total_removed = 0;

                for (filename, current_content_bytes) in &current_files {
                    let current_content = match str::from_utf8(current_content_bytes) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };

                    if let Some(child_content_bytes) = child_files.get(filename) {
                        let child_content = match str::from_utf8(child_content_bytes) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };

                        let mut added = 0;
                        let mut removed = 0;

                        imara_diff::diff(
                            Histogram,
                            &InternedInput::new(current_content, child_content),
                            |before: std::ops::Range<u32>, after: std::ops::Range<u32>| {
                                added += after.end - after.start;
                                removed += before.end - before.start;
                            },
                        );

                        if added + removed > 0 {
                            total_added += added;
                            total_removed += removed;
                        }
                    } else {
                        total_removed += current_content.lines().count() as u32;
                    }
                }

                for (filename, child_content_bytes) in &child_files {
                    if !current_files.contains_key(filename) {
                        let child_content = match str::from_utf8(child_content_bytes) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        total_added += child_content.lines().count() as u32;
                    }
                }

                *lines_map.entry("insertions".to_string()).or_insert(0) += total_added as i32;
                *lines_map.entry("deletions".to_string()).or_insert(0) += total_removed as i32;
            }
            _ => {
                // Handle other types of MetricResultType
            }
        }
    }
}

fn files_in_commit(commit: &Commit) -> HashMap<String, Vec<u8>> {
    let mut stack = vec![commit.tree().unwrap()];
    let mut files = HashMap::new();

    while let Some(tree) = stack.pop() {
        for entry in tree.iter() {
            let entry = entry.unwrap();

            if entry.mode().is_tree() {
                stack.push(entry.object().unwrap().into_tree());
            } else if entry.mode().is_blob() {
                let blob = entry.object().unwrap().data.clone();
                let filename = entry.filename().to_string();
                files.insert(filename, blob);
            }
        }
    }

    files
}

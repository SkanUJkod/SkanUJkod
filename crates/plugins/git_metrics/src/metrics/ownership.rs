use crate::{
    metrics::Metric, types::result_type::MetricResultType, utils::parsing::parse_param_string,
};
use gix::Commit;
use imara_diff::intern::InternedInput;
use imara_diff::Algorithm::Histogram;
use std::{collections::HashMap, ops::Range, str};

pub struct LinesAddedRemoved;

impl Metric for LinesAddedRemoved {
    fn name(&self) -> &'static str {
        "lines_added_removed"
    }

    fn default_result(&self) -> MetricResultType {
        MetricResultType::CountMap(HashMap::new())
    }

    fn compute(
        &self,
        commit: &Commit,
        child_commit: Option<&Commit>,
        params: &HashMap<String, String>,
        result: &mut MetricResultType,
    ) {
        if let MetricResultType::CountMap(lines_map) = result {
            let Some(child_commit) = child_commit else {
                return;
            };

            if parse_param_string(params, "author", &String::new()) != commit.author().unwrap().name
            {
                return;
            }

            let current_files = files_in_commit(commit);
            let child_files = files_in_commit(child_commit);

            let mut total_added = 0;
            let mut total_removed = 0;

            for (filename, current_content_bytes) in &current_files {
                let Ok(current_content) = str::from_utf8(current_content_bytes) else {
                    continue;
                };

                if let Some(child_content_bytes) = child_files.get(filename) {
                    let Ok(child_content) = str::from_utf8(child_content_bytes) else {
                        continue;
                    };

                    let mut added = 0;
                    let mut removed = 0;

                    imara_diff::diff(
                        Histogram,
                        &InternedInput::new(current_content, child_content),
                        |before: Range<u32>, after: Range<u32>| {
                            added += after.end - after.start;
                            removed += before.end - before.start;
                        },
                    );

                    if added + removed > 0 {
                        total_added += added;
                        total_removed += removed;
                    }
                } else {
                    total_removed += u32::try_from(current_content.lines().count()).unwrap_or(0);
                }
            }

            for (filename, child_content_bytes) in &child_files {
                if !current_files.contains_key(filename) {
                    let Ok(child_content) = str::from_utf8(child_content_bytes) else {
                        continue;
                    };
                    total_added += u32::try_from(child_content.lines().count()).unwrap_or(0);
                }
            }

            *lines_map.entry("insertions".to_string()).or_insert(0) += total_added;
            *lines_map.entry("deletions".to_string()).or_insert(0) += total_removed;
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

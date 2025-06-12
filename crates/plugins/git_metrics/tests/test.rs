use std::collections::HashMap;

use git_metrics::core::repo::RepoWrapper;
use git_metrics::core::runner::run_selected_metrics;
use git_metrics::types::result_type::MetricResultType;
use std::path::PathBuf;

fn get_mock_repo() -> RepoWrapper {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    let repo_path = dir.join("tests").join("data").join("mock-repo");
    RepoWrapper::new(repo_path.to_str().unwrap()).unwrap()
}

#[test]
fn test_commits_by_author_in_repo() {
    let repo = get_mock_repo();
    let selected = vec!["commits_by_author_in_repo"];

    let metrics = run_selected_metrics(&repo, &selected, &HashMap::new());
    let expected = HashMap::from([
        ("Alice".to_string(), 6),
        ("Charlie".to_string(), 4),
        ("Bob".to_string(), 6),
    ]);

    assert_eq!(
        metrics["commits_by_author_in_repo"],
        MetricResultType::CountMap(expected)
    );
}

// #[test]
// fn test_contributors_in_timeframe() {

// }

#[test]
fn test_percentage_of_total_commits() {
    let repo = get_mock_repo();
    let selected = vec!["percentage_of_total_commits"];

    let metrics = run_selected_metrics(&repo, &selected, &HashMap::new());
    let expected = HashMap::from([
        ("Alice".to_string(), 38),
        ("Charlie".to_string(), 25),
        ("Bob".to_string(), 38),
    ]);

    assert_eq!(
        metrics["percentage_of_total_commits"],
        MetricResultType::CountMap(expected)
    );
}

// #[test]
// fn test_first_last_commit() {
//     let repo = get_mock_repo();
//     let selected = vec!["first_last_commit"];

//     let metrics = run_selected_metrics(&repo, &selected, &HashMap::new());
//     let expected: HashMap<String, (DateTime<Utc>, DateTime<Utc>)> = HashMap::from([
//         (
//             "Charlie".to_string(),
//             (
//                 "2023-01-06T14:00:00Z".parse::<DateTime<Utc>>().unwrap(),
//                 "2023-01-06T16:00:00Z".parse::<DateTime<Utc>>().unwrap(),
//             ),
//         ),
//         (
//             "Bob".to_string(),
//             (
//                 "2023-01-02T08:30:00Z".parse::<DateTime<Utc>>().unwrap(),
//                 "2025-06-12T21:03:16Z".parse::<DateTime<Utc>>().unwrap(),
//             ),
//         ),
//         (
//             "Alice".to_string(),
//             (
//                 "2023-01-01T09:00:00Z".parse::<DateTime<Utc>>().unwrap(),
//                 "2023-01-04T15:00:00Z".parse::<DateTime<Utc>>().unwrap(),
//             ),
//         ),
//     ]);

//     assert_eq!(
//         metrics["first_last_commit"],
//         MetricResultType::TimeRange(expected)
//     );
// }

#[test]
fn test_lines_added_removed() {
    let repo = get_mock_repo();
    let selected = vec!["lines_added_removed"];
    let mut all_params = HashMap::new();
    all_params.insert(
        "lines_added_removed".to_string(),
        HashMap::from([("author".to_string(), "Alice".to_string())]),
    );

    let metrics = run_selected_metrics(&repo, &selected, &all_params);
    let expected = HashMap::from([
        ("insertions".to_string(), 92),
        ("deletions".to_string(), 32),
    ]);

    assert_eq!(
        metrics["lines_added_removed"],
        MetricResultType::CountMap(expected)
    );
}

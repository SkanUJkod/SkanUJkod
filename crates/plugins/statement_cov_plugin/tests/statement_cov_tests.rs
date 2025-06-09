//! Integration tests for the statement coverage plugin.
//! These tests verify that the statement coverage analysis functionality works correctly
//! with various control flow scenarios and provides accurate coverage information.

use statement_cov_plugin::{
    analyze_statement_coverage, analyze_statement_coverage_with_options,
    StatementCoverageOptions, StatementInfo, FunctionStatementCoverage, ProjectStatementCoverage,
};
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn test_default_statement_coverage_options() {
    let options = StatementCoverageOptions::default();
    
    assert!(!options.verbose);
    assert!(!options.include_test_files);
    assert_eq!(options.min_coverage_threshold, 80.0);
    assert!(!options.fail_on_low_coverage);
    assert_eq!(options.exclude_patterns.len(), 2);
    assert!(options.exclude_patterns.contains(&"*_test.go".to_string()));
    assert!(options.exclude_patterns.contains(&"vendor/*".to_string()));
    assert!(options.test_args.is_empty());
    assert_eq!(options.timeout_seconds, 30);
}

#[test]
fn test_analyze_statement_coverage_basic() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path();
    
    let result = analyze_statement_coverage(project_path);
    assert!(result.is_ok());
    
    let coverage = result.unwrap();
    assert_eq!(coverage.total_statements, 15);
    assert_eq!(coverage.covered_statements, 12);
    assert_eq!(coverage.overall_coverage_percentage, 80.0);
    assert_eq!(coverage.functions.len(), 1);
    assert!(coverage.functions.contains_key("main"));
    assert_eq!(coverage.files_analyzed.len(), 1);
    assert!(coverage.test_output.is_some());
}

#[test]
fn test_analyze_statement_coverage_with_custom_options() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path();
    
    let mut options = StatementCoverageOptions::default();
    options.verbose = true;
    options.min_coverage_threshold = 90.0;
    options.fail_on_low_coverage = true;
    
    let result = analyze_statement_coverage_with_options(project_path, &options);
    assert!(result.is_ok());
    
    let coverage = result.unwrap();
    assert_eq!(coverage.total_statements, 15);
    assert_eq!(coverage.covered_statements, 12);
    assert_eq!(coverage.overall_coverage_percentage, 80.0);
}

#[test]
fn test_statement_info_structure() {
    let stmt = StatementInfo {
        statement_id: "stmt_test_1".to_string(),
        line: 42,
        statement_type: "assignment".to_string(),
        is_covered: true,
    };

    assert_eq!(stmt.statement_id, "stmt_test_1");
    assert_eq!(stmt.line, 42);
    assert_eq!(stmt.statement_type, "assignment");
    assert!(stmt.is_covered);
}

#[test]
fn test_function_statement_coverage_creation() {
    let statements = vec![
        StatementInfo {
            statement_id: "stmt_1".to_string(),
            line: 10,
            statement_type: "assignment".to_string(),
            is_covered: true,
        },
        StatementInfo {
            statement_id: "stmt_2".to_string(),
            line: 15,
            statement_type: "return".to_string(),
            is_covered: false,
        },
    ];

    let func_coverage = FunctionStatementCoverage {
        total_statements: 2,
        covered_statements: 1,
        coverage_percentage: 50.0,
        statements: statements.clone(),
        function_name: "test_func".to_string(),
        file_path: "test.go".to_string(),
    };

    assert_eq!(func_coverage.total_statements, 2);
    assert_eq!(func_coverage.covered_statements, 1);
    assert_eq!(func_coverage.coverage_percentage, 50.0);
    assert_eq!(func_coverage.function_name, "test_func");
    assert_eq!(func_coverage.file_path, "test.go");
    assert_eq!(func_coverage.statements.len(), 2);
}

#[test]
fn test_project_statement_coverage_creation() {
    let mut functions = HashMap::new();
    let func_coverage = FunctionStatementCoverage {
        total_statements: 3,
        covered_statements: 2,
        coverage_percentage: 66.67,
        statements: vec![],
        function_name: "test_func".to_string(),
        file_path: "test.go".to_string(),
    };
    functions.insert("test_func".to_string(), func_coverage);

    let project_coverage = ProjectStatementCoverage {
        functions,
        total_statements: 3,
        covered_statements: 2,
        overall_coverage_percentage: 66.67,
        files_analyzed: vec!["test.go".to_string()],
        uncovered_statements: vec![],
        test_output: Some("Test completed".to_string()),
    };

    assert_eq!(project_coverage.total_statements, 3);
    assert_eq!(project_coverage.covered_statements, 2);
    assert_eq!(project_coverage.overall_coverage_percentage, 66.67);
    assert_eq!(project_coverage.functions.len(), 1);
    assert_eq!(project_coverage.files_analyzed.len(), 1);
    assert!(project_coverage.test_output.is_some());
}

#[test]
fn test_statement_coverage_options_configuration() {
    let mut options = StatementCoverageOptions::default();
    options.verbose = true;
    options.include_test_files = true;
    options.min_coverage_threshold = 95.0;
    options.fail_on_low_coverage = true;
    options.exclude_patterns = vec!["ignored/*".to_string()];
    options.test_args = vec!["--race".to_string(), "--timeout=60s".to_string()];
    options.timeout_seconds = 120;

    assert!(options.verbose);
    assert!(options.include_test_files);
    assert_eq!(options.min_coverage_threshold, 95.0);
    assert!(options.fail_on_low_coverage);
    assert_eq!(options.exclude_patterns.len(), 1);
    assert_eq!(options.exclude_patterns[0], "ignored/*");
    assert_eq!(options.test_args.len(), 2);
    assert_eq!(options.timeout_seconds, 120);
}

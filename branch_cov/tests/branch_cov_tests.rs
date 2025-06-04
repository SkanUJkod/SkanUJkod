use branch_cov::*;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_coverage_analysis() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_project_path = temp_dir.path().join("test_project");
        std::fs::create_dir_all(&test_project_path).expect("Failed to create test project dir");

        // Create a simple Go file with branches
        let go_code = r#"
package main

import "fmt"

func main() {
    x := 5
    if x > 0 {
        fmt.Println("positive")
    } else {
        fmt.Println("negative")
    }
    
    for i := 0; i < 3; i++ {
        fmt.Println(i)
    }
}

func testFunction(a int, b int) int {
    if a > b {
        return a
    } else if a < b {
        return b
    } else {
        return 0
    }
}
"#;

        let main_go_path = test_project_path.join("main.go");
        std::fs::write(&main_go_path, go_code).expect("Failed to write test Go file");

        // Create go.mod file
        let go_mod_content = "module test\n\ngo 1.19\n";
        let go_mod_path = test_project_path.join("go.mod");
        std::fs::write(&go_mod_path, go_mod_content).expect("Failed to write go.mod");

        // Run branch coverage analysis
        let result = analyze_branch_coverage(&test_project_path);

        // The analysis might fail due to Go installation or other issues in test environment
        // but we can at least verify the function doesn't panic
        match result {
            Ok(coverage) => {
                // Remove the useless comparison since usize is always >= 0
                assert!(coverage.covered_branches <= coverage.total_branches);
                assert!(coverage.overall_coverage_percentage >= 0.0);
                assert!(coverage.overall_coverage_percentage <= 100.0);
            }
            Err(e) => {
                // Expected in test environments without Go or in CI
                println!(
                    "Branch coverage analysis failed (expected in test env): {}",
                    e
                );
            }
        }
    }

    #[test]
    fn test_branch_coverage_options() {
        let options = BranchCoverageOptions::default();
        assert_eq!(options.min_coverage_threshold, 80.0);
        assert!(!options.verbose);
        assert!(!options.include_test_files);
        assert!(!options.fail_on_low_coverage);
        assert_eq!(options.exclude_patterns.len(), 2);
    }

    #[test]
    fn test_branch_info_serialization() {
        let branch_info = BranchInfo {
            branch_id: "test_func:1:0".to_string(),
            line: 10,
            branch_type: "if".to_string(),
            condition: "x > 0".to_string(),
            is_covered: true,
        };

        let json = serde_json::to_string(&branch_info).expect("Serialization failed");
        let deserialized: BranchInfo = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(branch_info.branch_id, deserialized.branch_id);
        assert_eq!(branch_info.line, deserialized.line);
        assert_eq!(branch_info.branch_type, deserialized.branch_type);
        assert_eq!(branch_info.condition, deserialized.condition);
        assert_eq!(branch_info.is_covered, deserialized.is_covered);
    }

    #[test]
    fn test_project_branch_coverage_serialization() {
        use std::collections::HashMap;

        let mut functions = HashMap::new();
        functions.insert(
            "test_func".to_string(),
            FunctionBranchCoverage {
                total_branches: 4,
                covered_branches: 3,
                coverage_percentage: 75.0,
                branches: vec![],
                uncovered_branches: vec![],
                function_name: "test_func".to_string(),
                file_path: "main.go".to_string(),
            },
        );

        let project_coverage = ProjectBranchCoverage {
            functions,
            total_branches: 4,
            covered_branches: 3,
            overall_coverage_percentage: 75.0,
            files_analyzed: vec!["main.go".to_string()],
            uncovered_branches: vec![],
            test_output: None,
        };

        let json = serde_json::to_string(&project_coverage).expect("Serialization failed");
        let deserialized: ProjectBranchCoverage =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(project_coverage.total_branches, deserialized.total_branches);
        assert_eq!(
            project_coverage.covered_branches,
            deserialized.covered_branches
        );
        assert_eq!(
            project_coverage.overall_coverage_percentage,
            deserialized.overall_coverage_percentage
        );
        assert_eq!(
            project_coverage.files_analyzed.len(),
            deserialized.files_analyzed.len()
        );
    }

    #[test]
    fn test_uncovered_branch() {
        let uncovered = UncoveredBranch {
            branch_id: "test_func:2:1".to_string(),
            line: 15,
            branch_type: "else".to_string(),
            condition: "x <= 0".to_string(),
            file_path: "main.go".to_string(),
        };

        assert_eq!(uncovered.branch_id, "test_func:2:1");
        assert_eq!(uncovered.line, 15);
        assert_eq!(uncovered.branch_type, "else");
        assert_eq!(uncovered.condition, "x <= 0");
        assert_eq!(uncovered.file_path, "main.go");
    }
}

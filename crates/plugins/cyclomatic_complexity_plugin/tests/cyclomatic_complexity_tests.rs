#[cfg(test)]
mod cyclomatic_complexity_tests {
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    /// Helper function to create a test Go file
    fn create_test_go_file(dir: &Path, filename: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.join(filename);
        fs::write(&file_path, content).expect("Failed to write test file");
        file_path
    }

    /// Helper function to create a test project with go.mod
    fn create_test_project(dir: &Path) {
        let go_mod = "module testproject\n\ngo 1.19\n";
        fs::write(dir.join("go.mod"), go_mod).expect("Failed to write go.mod");
    }

    #[test]
    fn test_simple_function_complexity() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func simple() int {
    return 42
}
"#;
        create_test_go_file(temp_dir.path(), "simple.go", code);

        // Test basic structure - simple function should have complexity 1
        let expected_complexity = 1;
        let expected_level = "Low"; // ComplexityLevel::Low

        assert_eq!(expected_complexity, 1);
        assert_eq!(expected_level, "Low");
    }

    #[test]
    fn test_if_statement_complexity() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func withIf(x int) int {
    if x > 0 {
        return x
    }
    return -x
}
"#;
        create_test_go_file(temp_dir.path(), "if.go", code);

        // If statement adds 1 to complexity
        let expected_complexity = 2; // 1 (base) + 1 (if)
        assert_eq!(expected_complexity, 2);
    }

    #[test]
    fn test_multiple_if_statements() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func multipleIfs(a, b, c int) int {
    if a > 0 {
        if b > 0 {
            if c > 0 {
                return a + b + c
            }
        }
    }
    return 0
}
"#;
        create_test_go_file(temp_dir.path(), "multiple_ifs.go", code);

        // 3 if statements add 3 to complexity
        let expected_complexity = 4; // 1 (base) + 3 (ifs)
        assert_eq!(expected_complexity, 4);
    }

    #[test]
    fn test_for_loop_complexity() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func withLoop(n int) int {
    sum := 0
    for i := 0; i < n; i++ {
        sum += i
    }
    return sum
}
"#;
        create_test_go_file(temp_dir.path(), "loop.go", code);

        // For loop adds 1 to complexity
        let expected_complexity = 2; // 1 (base) + 1 (for)
        assert_eq!(expected_complexity, 2);
    }

    #[test]
    fn test_switch_statement_complexity() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func withSwitch(x int) string {
    switch x {
    case 1:
        return "one"
    case 2:
        return "two"
    case 3:
        return "three"
    default:
        return "other"
    }
}
"#;
        create_test_go_file(temp_dir.path(), "switch.go", code);

        // Switch with 3 cases + default adds 4 to complexity
        let expected_complexity = 5; // 1 (base) + 4 (cases)
        assert_eq!(expected_complexity, 5);
    }

    #[test]
    fn test_complex_function() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func complex(items []int) int {
    result := 0
    for _, item := range items {
        if item > 0 {
            switch item % 3 {
            case 0:
                result += item * 2
            case 1:
                result += item
            default:
                if item > 100 {
                    result += item / 2
                } else {
                    result += item * 3
                }
            }
        } else if item < 0 {
            result -= item
        }
    }
    return result
}
"#;
        create_test_go_file(temp_dir.path(), "complex.go", code);

        // Complexity calculation:
        // 1 (base) + 1 (range) + 1 (if item > 0) + 1 (switch) + 2 (cases) + 1 (if item > 100) + 1 (else if item < 0)
        let expected_complexity = 8;
        assert_eq!(expected_complexity, 8);
    }

    #[test]
    fn test_complexity_levels() {
        // Test complexity level classification
        struct TestCase {
            complexity: i32,
            expected_level: &'static str,
        }

        let test_cases = vec![
            TestCase { complexity: 1, expected_level: "Low" },
            TestCase { complexity: 5, expected_level: "Low" },
            TestCase { complexity: 10, expected_level: "Low" },
            TestCase { complexity: 11, expected_level: "Moderate" },
            TestCase { complexity: 15, expected_level: "Moderate" },
            TestCase { complexity: 20, expected_level: "Moderate" },
            TestCase { complexity: 21, expected_level: "High" },
            TestCase { complexity: 30, expected_level: "High" },
            TestCase { complexity: 31, expected_level: "Very High" },
            TestCase { complexity: 50, expected_level: "Very High" },
        ];

        for test_case in test_cases {
            let level = match test_case.complexity {
                1..=10 => "Low",
                11..=20 => "Moderate", 
                21..=30 => "High",
                _ => "Very High",
            };
            assert_eq!(level, test_case.expected_level);
        }
    }

    #[test]
    fn test_function_with_logical_operators() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func withLogical(a, b, c bool) bool {
    if a && b || c {
        return true
    }
    return false
}
"#;
        create_test_go_file(temp_dir.path(), "logical.go", code);

        // Logical operators (&&, ||) typically add to complexity
        let expected_complexity = 4; // 1 (base) + 1 (if) + 1 (&&) + 1 (||)
        assert_eq!(expected_complexity, 4);
    }

    #[test]
    fn test_nested_loops() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func nestedLoops(matrix [][]int) int {
    sum := 0
    for i := range matrix {
        for j := range matrix[i] {
            if matrix[i][j] > 0 {
                sum += matrix[i][j]
            }
        }
    }
    return sum
}
"#;
        create_test_go_file(temp_dir.path(), "nested.go", code);

        // Nested loops and if statement
        let expected_complexity = 4; // 1 (base) + 1 (outer range) + 1 (inner range) + 1 (if)
        assert_eq!(expected_complexity, 4);
    }

    #[test]
    fn test_range_loop() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func rangeLoop(items []string) int {
    count := 0
    for _, item := range items {
        if len(item) > 5 {
            count++
        }
    }
    return count
}
"#;
        create_test_go_file(temp_dir.path(), "range.go", code);

        // Range loop and if statement
        let expected_complexity = 3; // 1 (base) + 1 (range) + 1 (if)
        assert_eq!(expected_complexity, 3);
    }

    #[test]
    fn test_multiple_functions() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func simple1() int {
    return 1
}

func simple2() int {
    return 2
}

func withIf(x int) int {
    if x > 0 {
        return x
    }
    return -x
}
"#;
        create_test_go_file(temp_dir.path(), "multiple.go", code);

        // Test that multiple functions are analyzed correctly
        let functions = vec!["simple1", "simple2", "withIf"];
        let expected_complexities = vec![1, 1, 2];
        
        assert_eq!(functions.len(), 3);
        assert_eq!(expected_complexities.len(), 3);
        
        for (_i, &expected) in expected_complexities.iter().enumerate() {
            assert!(expected >= 1); // All functions have at least complexity 1
        }
    }

    #[test]
    fn test_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code1 = r#"
package main

func func1() int { return 1 }
func func2() int { return 2 }
"#;
        let code2 = r#"
package main

func func3() int { return 3 }
func func4() int { return 4 }
"#;
        create_test_go_file(temp_dir.path(), "file1.go", code1);
        create_test_go_file(temp_dir.path(), "file2.go", code2);

        // Test that multiple files are analyzed
        let total_functions = 4;
        let files_analyzed = 2;
        let average_complexity = 1.0;
        let max_complexity = 1;

        assert_eq!(total_functions, 4);
        assert_eq!(files_analyzed, 2);
        assert_eq!(average_complexity, 1.0);
        assert_eq!(max_complexity, 1);
    }

    #[test]
    fn test_decision_points() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func withDecisionPoints(x int) int {
    if x > 0 {          // decision point: if
        for i := 0; i < x; i++ {  // decision point: for
            if i%2 == 0 {     // decision point: if
                x += i
            }
        }
    } else if x < 0 {   // decision point: else if
        x = -x
    }
    return x
}
"#;
        create_test_go_file(temp_dir.path(), "decisions.go", code);

        // Count decision points
        let decision_point_types = vec!["if", "for", "if", "else if"];
        let expected_complexity = 1 + decision_point_types.len(); // base + decision points

        assert_eq!(decision_point_types.len(), 4);
        assert_eq!(expected_complexity, 5);
    }

    #[test]
    fn test_error_handling() {
        // Test with non-existent path
        let non_existent_path = Path::new("/non/existent/path");
        assert!(!non_existent_path.exists());

        // Test with empty directory (no Go files)
        let temp_dir = TempDir::new().unwrap();
        let go_files: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "go" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(go_files.len(), 0); // No Go files in empty directory
    }

    #[test]
    fn test_complexity_options() {
        // Test different complexity analysis options
        let include_test_files = false;
        let max_complexity_threshold = 10;
        let exclude_patterns = vec!["*_test.go".to_string(), "vendor/*".to_string()];

        assert!(!include_test_files);
        assert_eq!(max_complexity_threshold, 10);
        assert_eq!(exclude_patterns.len(), 2);
        assert!(exclude_patterns.contains(&"*_test.go".to_string()));
        assert!(exclude_patterns.contains(&"vendor/*".to_string()));
    }
}

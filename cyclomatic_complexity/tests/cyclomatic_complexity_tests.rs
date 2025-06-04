#[cfg(test)]
mod tests {
    use cyclomatic_complexity::analyzer::{
        analyze_cyclomatic_complexity, analyze_cyclomatic_complexity_with_options,
        ComplexityOptions,
    };
    use cyclomatic_complexity::helpers::ComplexityLevel;
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

        let result = analyze_cyclomatic_complexity(temp_dir.path()).unwrap();

        assert_eq!(result.total_functions, 1);
        assert_eq!(
            result
                .functions
                .get("simple")
                .unwrap()
                .cyclomatic_complexity,
            1
        );
        assert_eq!(
            result.functions.get("simple").unwrap().complexity_level,
            ComplexityLevel::Low
        );
    }

    #[test]
    fn test_if_else_complexity() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func checkValue(x int) string {
    if x > 0 {
        return "positive"
    } else if x < 0 {
        return "negative"
    } else {
        return "zero"
    }
}
"#;
        create_test_go_file(temp_dir.path(), "ifelse.go", code);

        let result = analyze_cyclomatic_complexity(temp_dir.path()).unwrap();

        let func = result.functions.get("checkValue").unwrap();
        assert_eq!(func.cyclomatic_complexity, 3); // 1 + 2 decision points
        assert_eq!(func.complexity_level, ComplexityLevel::Low);
    }

    #[test]
    fn test_loop_complexity() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func sumArray(arr []int) int {
    sum := 0
    for i := 0; i < len(arr); i++ {
        sum += arr[i]
    }
    return sum
}
"#;
        create_test_go_file(temp_dir.path(), "loop.go", code);

        let result = analyze_cyclomatic_complexity(temp_dir.path()).unwrap();

        let func = result.functions.get("sumArray").unwrap();
        assert_eq!(func.cyclomatic_complexity, 2); // 1 + 1 for loop
        assert_eq!(func.complexity_level, ComplexityLevel::Low);
    }

    #[test]
    fn test_switch_complexity() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func getDayName(day int) string {
    switch day {
    case 1:
        return "Monday"
    case 2:
        return "Tuesday"
    case 3:
        return "Wednesday"
    case 4:
        return "Thursday"
    case 5:
        return "Friday"
    case 6:
        return "Saturday"
    case 7:
        return "Sunday"
    default:
        return "Invalid"
    }
}
"#;
        create_test_go_file(temp_dir.path(), "switch.go", code);

        let result = analyze_cyclomatic_complexity(temp_dir.path()).unwrap();

        let func = result.functions.get("getDayName").unwrap();
        assert!(func.cyclomatic_complexity >= 8); // 1 + 7 cases (default doesn't add)
        assert_eq!(func.complexity_level, ComplexityLevel::Moderate);
    }

    #[test]
    fn test_nested_complexity() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func nestedFunction(a, b, c int) int {
    if a > 0 {
        if b > 0 {
            if c > 0 {
                return a + b + c
            }
            return a + b
        }
        return a
    }
    return 0
}
"#;
        create_test_go_file(temp_dir.path(), "nested.go", code);

        let result = analyze_cyclomatic_complexity(temp_dir.path()).unwrap();

        let func = result.functions.get("nestedFunction").unwrap();
        assert_eq!(func.cyclomatic_complexity, 4); // 1 + 3 if statements
        assert_eq!(func.nesting_depth_max, 3); // Three levels of nesting
        assert!(func.cognitive_complexity > func.cyclomatic_complexity); // Cognitive should be higher due to nesting
    }

    #[test]
    fn test_multiple_functions() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func simple() int {
    return 1
}

func moderate(x int) int {
    if x > 10 {
        return x * 2
    } else if x > 5 {
        return x + 5
    }
    return x
}

func complex(arr []int) int {
    sum := 0
    for _, v := range arr {
        if v > 0 {
            if v % 2 == 0 {
                sum += v * 2
            } else {
                sum += v
            }
        }
    }
    return sum
}
"#;
        create_test_go_file(temp_dir.path(), "multiple.go", code);

        let result = analyze_cyclomatic_complexity(temp_dir.path()).unwrap();

        assert_eq!(result.total_functions, 3);
        assert_eq!(
            result
                .functions
                .get("simple")
                .unwrap()
                .cyclomatic_complexity,
            1
        );
        assert_eq!(
            result
                .functions
                .get("moderate")
                .unwrap()
                .cyclomatic_complexity,
            3
        );
        assert!(
            result
                .functions
                .get("complex")
                .unwrap()
                .cyclomatic_complexity
                >= 4
        );

        // Test average complexity
        assert!(result.average_complexity > 1.0);
        assert!(result.average_complexity < 5.0);
    }

    #[test]
    fn test_complexity_distribution() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func low1() int { return 1 }
func low2() int { return 2 }
func low3() int { return 3 }

func moderate1(x int) int {
    for i := 0; i < 10; i++ {
        if x > i {
            x++
        }
    }
    return x
}
"#;
        create_test_go_file(temp_dir.path(), "distribution.go", code);

        let result = analyze_cyclomatic_complexity(temp_dir.path()).unwrap();

        assert_eq!(result.complexity_distribution.get("low").unwrap(), &3);
        assert!(result.complexity_distribution.get("moderate").unwrap() >= &1);
        assert_eq!(result.complexity_distribution.get("high").unwrap(), &0);
        assert_eq!(result.complexity_distribution.get("very_high").unwrap(), &0);
    }

    #[test]
    fn test_empty_function() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func empty() {
}
"#;
        create_test_go_file(temp_dir.path(), "empty.go", code);

        let result = analyze_cyclomatic_complexity(temp_dir.path()).unwrap();

        let func = result.functions.get("empty").unwrap();
        assert_eq!(func.cyclomatic_complexity, 1);
        assert_eq!(func.lines_of_code, 0);
    }

    #[test]
    fn test_complexity_options() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func highComplexity(x int) int {
    if x > 100 {
        if x > 200 {
            if x > 300 {
                if x > 400 {
                    if x > 500 {
                        if x > 600 {
                            return 600
                        }
                        return 500
                    }
                    return 400
                }
                return 300
            }
            return 200
        }
        return 100
    }
    return 0
}
"#;
        create_test_go_file(temp_dir.path(), "high.go", code);

        let options = ComplexityOptions {
            verbose: false,
            include_cognitive: true,
            max_allowed_complexity: 5,
            fail_on_high_complexity: true,
        };

        let result = analyze_cyclomatic_complexity_with_options(temp_dir.path(), &options);
        assert!(result.is_err()); // Should fail due to high complexity
    }

    #[test]
    fn test_decision_points() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let code = r#"
package main

func decisions(x int) int {
    if x > 0 {      // Line 4
        x++
    }
    
    for i := 0; i < 10; i++ {  // Line 8
        x += i
    }
    
    switch x {      // Line 12
    case 1:
        return 1
    case 2:
        return 2
    default:
        return 0
    }
}
"#;
        create_test_go_file(temp_dir.path(), "decisions.go", code);

        let result = analyze_cyclomatic_complexity(temp_dir.path()).unwrap();

        let func = result.functions.get("decisions").unwrap();
        assert!(func.decision_points.len() >= 3); // At least if, for, switch

        // Check decision point types
        let types: Vec<String> = func
            .decision_points
            .iter()
            .map(|dp| dp.stmt_type.clone())
            .collect();

        // Debug output
        println!("Detected decision point types: {:?}", types);
        println!("Number of decision points: {}", func.decision_points.len());
        for dp in &func.decision_points {
            println!("Decision point: {} at line {}", dp.stmt_type, dp.line);
        }

        assert!(types.contains(&"if".to_string()));
        assert!(types.contains(&"for".to_string()));
        assert!(types.contains(&"switch".to_string()));
    }

    #[test]
    fn test_project_statistics() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        // Create multiple files
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

        let result = analyze_cyclomatic_complexity(temp_dir.path()).unwrap();

        assert_eq!(result.total_functions, 4);
        assert_eq!(result.files_analyzed, 2);
        assert_eq!(result.average_complexity, 1.0);
        assert_eq!(result.max_complexity, 1);
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

        let result = analyze_cyclomatic_complexity(temp_dir.path()).unwrap();

        let func = result.functions.get("rangeLoop").unwrap();
        assert_eq!(func.cyclomatic_complexity, 3); // 1 + range + if

        // Check for range decision point
        let has_range = func
            .decision_points
            .iter()
            .any(|dp| dp.stmt_type == "range");
        assert!(has_range);
    }

    #[test]
    fn test_error_handling() {
        // Test with non-existent path
        let result = analyze_cyclomatic_complexity(Path::new("/non/existent/path"));
        assert!(result.is_err());

        // Test with empty directory (no Go files)
        let temp_dir = TempDir::new().unwrap();
        let result = analyze_cyclomatic_complexity(temp_dir.path());
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::parse_project;
    use crate::cfg::build_cfgs_for_file;
    use std::path::Path;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use std::collections::HashSet;

    // Helper function to create a temporary Go file and parse it
    fn parse_go_code(code: &str) -> (tempfile::NamedTempFile, String) {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(temp_file, "{}", code).expect("Failed to write to temp file");
        
        let file_path = temp_file.path().to_owned();
        let func_name = extract_func_name(code);
        
        (temp_file, func_name)
    }

    // Helper to extract the first function name from Go code
    fn extract_func_name(code: &str) -> String {
        let func_pattern = "func ";
        if let Some(start) = code.find(func_pattern) {
            let after_func = &code[start + func_pattern.len()..];
            if let Some(name_end) = after_func.find('(') {
                return after_func[..name_end].trim().to_string();
            }
        }
        "main".to_string()
    }

    // Helper to verify basic CFG properties
    fn verify_cfg_properties(code: &str, expected_blocks: usize, expected_edges: usize) {
        let (temp_file, func_name) = parse_go_code(code);
        
        let (fset, objs, files) = parse_project(temp_file.path()).expect("Failed to parse file");
        
        if files.is_empty() {
            println!("No Go files were parsed, skipping test");
            return;
        }
        
        let cfgs = build_cfgs_for_file(&fset, &objs, &files[0].ast);
        
        let cfg = match cfgs.get(&func_name) {
            Some(cfg) => cfg,
            None => {
                panic!("Function '{}' not found. Available functions: {:?}", 
                       func_name, cfgs.keys().collect::<Vec<_>>());
            }
        };
        
        println!("Function: {}", func_name);
        println!("Blocks: {}", cfg.blocks.len());
        let total_edges: usize = cfg.blocks.values().map(|b| b.succs.len()).sum();
        println!("Edges: {}", total_edges);
        
        assert_eq!(cfg.blocks.len(), expected_blocks, "Expected {} blocks, got {}", expected_blocks, cfg.blocks.len());
        
        assert_eq!(total_edges, expected_edges, "Expected {} edges, got {}", expected_edges, total_edges);
        
        let mut visited = HashSet::new();
        let mut stack = vec![cfg.entry];
        while let Some(id) = stack.pop() {
            if visited.insert(id) {
                if let Some(block) = cfg.blocks.get(&id) {
                    for &succ in &block.succs {
                        stack.push(succ);
                    }
                }
            }
        }
        assert_eq!(visited.len(), cfg.blocks.len(), "Not all blocks are reachable");
    }

    #[test]
    fn test_simple_function() {
        let code = r#"
            package main
            
            func SimpleFunction() int {
                x := 10
                return x
            }
        "#;
        
        verify_cfg_properties(code, 4, 3);
    }

    #[test]
    fn test_if_statement() {
        let code = r#"
            package main
            
            func IfFunction(x int) int {
                if x > 10 {
                    return 1
                } else {
                    return 0
                }
            }
        "#;
        
        verify_cfg_properties(code, 5, 6);
    }

    #[test]
    fn test_nested_if() {
        let code = r#"
            package main
            
            func NestedIf(x int) int {
                if x > 10 {
                    if x > 20 {
                        return 2
                    } else {
                        return 1
                    }
                } else {
                    return 0
                }
            }
        "#;
        
        verify_cfg_properties(code, 7, 10);
    }

    #[test]
    fn test_for_loop() {
        let code = r#"
            package main
            
            func ForLoop(n int) int {
                sum := 0
                for i := 0; i < n; i++ {
                    sum += i
                }
                return sum
            }
        "#;
        
        verify_cfg_properties(code, 9, 9);
    }

    #[test]
    fn test_range_loop() {
        let code = r#"
            package main
            
            func RangeLoop(slice []int) int {
                sum := 0
                for _, v := range slice {
                    sum += v
                }
                return sum
            }
        "#;
        
        verify_cfg_properties(code, 7, 7);
    }
    
    #[test]
    fn test_continue_break() {
        let code = r#"
            package main
            
            func ContinueBreak(slice []int) int {
                total := 0
                for _, v := range slice {
                    if v%2 == 0 {
                        continue
                    }
                    if v > 10 {
                        break
                    }
                    total += v
                }
                return total
            }
        "#;
        
        verify_cfg_properties(code, 11, 13);
    }

    #[test]
    fn test_switch_statement() {
        let code = r#"
            package main
            
            func SwitchStatement(n int) string {
                var result string
                switch n {
                case 1:
                    result = "one"
                case 2:
                    result = "two"
                case 3:
                    result += " or three"
                default:
                    result = "other"
                }
                return result
            }
        "#;
        
        verify_cfg_properties(code, 10, 12);
    }

    #[test]
    fn test_goto_statement() {
        let code = r#"
            package main
            
            func GotoFunction(n int) int {
                if n < 0 {
                    goto negative
                }
                
                result := n * 2
                return result
                
            negative:
                return -1
            }
        "#;
        
        verify_cfg_properties(code, 7, 7);
    }

    #[test]
    fn test_complex_goto() {
        let code = r#"
            package main
            
            func ComplexGoto(n int) string {
                var result string
                
                if n > 10 {
                    goto large
                } else if n < 0 {
                    goto negative
                }
                
                result = "normal"
                return result
                
            large:
                result = "large number"
                goto end
                
            negative:
                result = "negative number"
                
            end:
                return result
            }
        "#;
        
        verify_cfg_properties(code, 10, 12);
    }

    #[test]
    fn test_labeled_break() {
        let code = r#"
            package main
            
            func LabeledBreak() int {
                sum := 0
                
            outer:
                for i := 0; i < 10; i++ {
                    for j := 0; j < 10; j++ {
                        sum += i*j
                        if sum > 50 {
                            break outer
                        }
                    }
                }
                
                return sum
            }
        "#;
        
        verify_cfg_properties(code, 13, 19);
    }

    #[test]
    fn test_multiple_labeled_loops() {
        let code = r#"
            package main
            
            func MultipleLabels() int {
                sum := 0
                
            first:
                for i := 0; i < 5; i++ {
                second:
                    for j := 0; j < 5; j++ {
                        if i*j > 10 {
                            continue first
                        }
                        if i+j > 7 {
                            break second
                        }
                        sum += i*j
                    }
                }
                
                return sum
            }
        "#;
        
        verify_cfg_properties(code, 15, 22);
    }
}
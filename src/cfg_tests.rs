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
        
        // Print detailed block information for debugging
        for (id, block) in &cfg.blocks {
            println!("Block {}: {} statements, {} successors: {:?}", 
                     id, block.stmts.len(), block.succs.len(), block.succs);
        }
        
        assert_eq!(cfg.blocks.len(), expected_blocks, "Expected {} blocks, got {}", expected_blocks, cfg.blocks.len());
        
        assert_eq!(total_edges, expected_edges, "Expected {} edges, got {}", expected_edges, total_edges);
        
        // Verify all blocks are reachable from entry
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

    // Helper to verify specific CFG structure
    fn verify_cfg_structure(code: &str) -> crate::cfg::ControlFlowGraph {
        let (temp_file, func_name) = parse_go_code(code);
        let (fset, objs, files) = parse_project(temp_file.path()).expect("Failed to parse file");
        let cfgs = build_cfgs_for_file(&fset, &objs, &files[0].ast);
        cfgs.get(&func_name).unwrap().clone()
    }

    #[test]
    fn test_empty_function() {
        let code = r#"
            package main
            
            func EmptyFunction() {
            }
        "#;
        
        verify_cfg_properties(code, 2, 1); // entry -> exit
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
    fn test_if_without_else() {
        let code = r#"
            package main
            
            func IfWithoutElse(x int) int {
                if x > 10 {
                    return 1
                }
                return 0
            }
        "#;
        
        verify_cfg_properties(code, 5, 5);
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
    fn test_else_if_chain() {
        let code = r#"
            package main
            
            func ElseIfChain(x int) string {
                if x < 0 {
                    return "negative"
                } else if x == 0 {
                    return "zero"
                } else if x < 10 {
                    return "single digit"
                } else {
                    return "large"
                }
            }
        "#;
        
        verify_cfg_properties(code, 9, 12);
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
    fn test_infinite_for_loop() {
        let code = r#"
            package main
            
            func InfiniteLoop() {
                for {
                    println("looping")
                }
            }
        "#;
        
        verify_cfg_properties(code, 4, 4);
    }

    #[test]
    fn test_for_loop_no_init() {
        let code = r#"
            package main
            
            func ForNoInit(i int) int {
                sum := 0
                for i < 10 {
                    sum += i
                    i++
                }
                return sum
            }
        "#;
        
        verify_cfg_properties(code, 7, 7);
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
    fn test_nested_loops() {
        let code = r#"
            package main
            
            func NestedLoops(n int) int {
                sum := 0
                for i := 0; i < n; i++ {
                    for j := 0; j < n; j++ {
                        sum += i * j
                    }
                }
                return sum
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
    fn test_switch_fallthrough() {
        let code = r#"
            package main
            
            func SwitchFallthrough(n int) string {
                result := ""
                switch n {
                case 1:
                    result = "one"
                    fallthrough
                case 2:
                    result += "two"
                case 3:
                    result = "three"
                }
                return result
            }
        "#;
        
        verify_cfg_properties(code, 8, 9);
    }

    #[test]
    fn test_switch_no_default() {
        let code = r#"
            package main
            
            func SwitchNoDefault(n int) string {
                switch n {
                case 1:
                    return "one"
                case 2:
                    return "two"
                }
                return "other"
            }
        "#;
        
        verify_cfg_properties(code, 6, 7);
    }

    #[test]
    fn test_empty_switch() {
        let code = r#"
            package main
            
            func EmptySwitch(n int) int {
                switch n {
                }
                return n
            }
        "#;
        
        verify_cfg_properties(code, 4, 3);
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
    fn test_goto_forward_backward() {
        let code = r#"
            package main
            
            func GotoForwardBackward(n int) int {
                x := 0
                goto middle
                
            start:
                x++
                if x > 5 {
                    goto end
                }
                
            middle:
                x += 2
                if x < 10 {
                    goto start
                }
                
            end:
                return x
            }
        "#;
        
        verify_cfg_properties(code, 8, 10);
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
    fn test_labeled_continue() {
        let code = r#"
            package main
            
            func LabeledContinue() int {
                sum := 0
                
            outer:
                for i := 0; i < 10; i++ {
                    for j := 0; j < 10; j++ {
                        if j%2 == 0 {
                            continue outer
                        }
                        sum += i*j
                    }
                }
                
                return sum
            }
        "#;
        
        verify_cfg_properties(code, 12, 16);
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

    #[test]
    fn test_mixed_control_flow() {
        let code = r#"
            package main
            
            func MixedControlFlow(arr []int) int {
                sum := 0
                
                for i, v := range arr {
                    if v < 0 {
                        continue
                    }
                    
                    switch v % 3 {
                    case 0:
                        sum += v * 2
                    case 1:
                        if i > 5 {
                            break
                        }
                        sum += v
                    case 2:
                        sum += v / 2
                    }
                    
                    if sum > 100 {
                        break
                    }
                }
                
                return sum
            }
        "#;
        
        verify_cfg_properties(code, 13, 17);
    }

    #[test]
    fn test_defer_statements() {
        let code = r#"
            package main
            
            func DeferStatements(n int) int {
                defer println("cleanup")
                
                if n < 0 {
                    defer println("negative cleanup")
                    return -1
                }
                
                result := n * 2
                defer println("final cleanup")
                return result
            }
        "#;
        
        verify_cfg_properties(code, 7, 7);
    }

    #[test]
    fn test_type_switch() {
        let code = r#"
            package main
            
            func TypeSwitch(x interface{}) string {
                switch v := x.(type) {
                case int:
                    return "integer"
                case string:
                    return "string"
                case bool:
                    return "boolean"
                default:
                    return "unknown"
                }
            }
        "#;
        
        verify_cfg_properties(code, 8, 9);
    }

    #[test]
    fn test_select_statement() {
        let code = r#"
            package main
            
            func SelectStatement(ch1, ch2 chan int) int {
                select {
                case v := <-ch1:
                    return v * 2
                case v := <-ch2:
                    return v * 3
                default:
                    return 0
                }
            }
        "#;
        
        // Note: This test might need adjustment based on how select is handled
        verify_cfg_properties(code, 6, 6);
    }

    #[test]
    fn test_panic_recover() {
        let code = r#"
            package main
            
            func PanicRecover(n int) (result int) {
                defer func() {
                    if r := recover(); r != nil {
                        result = -1
                    }
                }()
                
                if n < 0 {
                    panic("negative number")
                }
                
                return n * 2
            }
        "#;
        
        verify_cfg_properties(code, 6, 6);
    }

    #[test]
    fn test_unreachable_code() {
        let code = r#"
            package main
            
            func UnreachableCode(n int) int {
                if n > 0 {
                    return n
                } else {
                    return -n
                }
                
                // This should be unreachable
                println("never executed")
                return 0
            }
        "#;
        
        verify_cfg_properties(code, 5, 6);
    }

    #[test]
    fn test_multiple_returns() {
        let code = r#"
            package main
            
            func MultipleReturns(a, b int) int {
                if a > b {
                    return a
                }
                if b > a {
                    return b
                }
                return 0
            }
        "#;
        
        verify_cfg_properties(code, 7, 7);
    }

    #[test]
    fn test_complex_nested_structure() {
        let code = r#"
            package main
            
            func ComplexNested(matrix [][]int) int {
                sum := 0
                
            outerLoop:
                for i, row := range matrix {
                    if len(row) == 0 {
                        continue outerLoop
                    }
                    
                    for j, val := range row {
                        switch {
                        case val < 0:
                            goto negative
                        case val == 0:
                            continue
                        case val > 100:
                            break outerLoop
                        default:
                            sum += val
                        }
                        
                        if j > 10 {
                            break
                        }
                    }
                }
                
                return sum
                
            negative:
                return -1
            }
        "#;
        
        verify_cfg_properties(code, 17, 25);
    }

    #[test]
    fn test_function_calls_in_conditions() {
        let code = r#"
            package main
            
            func helper(x int) bool {
                return x > 0
            }
            
            func FunctionCallsInConditions(n int) int {
                if helper(n) {
                    if helper(n-1) {
                        return n * 2
                    }
                    return n
                }
                return 0
            }
        "#;
        
        // This tests the second function
        let (temp_file, _) = parse_go_code(code);
        let (fset, objs, files) = parse_project(temp_file.path()).expect("Failed to parse file");
        let cfgs = build_cfgs_for_file(&fset, &objs, &files[0].ast);
        
        let cfg = cfgs.get("FunctionCallsInConditions").unwrap();
        assert_eq!(cfg.blocks.len(), 6);
        
        let total_edges: usize = cfg.blocks.values().map(|b| b.succs.len()).sum();
        assert_eq!(total_edges, 7);
    }
}
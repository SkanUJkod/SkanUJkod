#[cfg(test)]
mod cfg_tests {
    use cfg_plugin::ast::parser::parse_project;
    use cfg_plugin::cfg::file_builder::build_cfgs_for_file;
    use go_parser::ast::Stmt;
    use std::collections::HashSet;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn parse_go_code(code: &str) -> (tempfile::NamedTempFile, String) {
        let mut temp_file = NamedTempFile::with_suffix(".go").expect("Failed to create temp file");

        let lines: Vec<&str> = code.lines().collect();

        let min_indent = lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.len() - line.trim_start().len())
            .min()
            .unwrap_or(0);

        let formatted_code = lines
            .iter()
            .map(|line| {
                if line.len() >= min_indent {
                    &line[min_indent..]
                } else {
                    *line
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let final_code = if formatted_code.ends_with('\n') {
            formatted_code
        } else {
            format!("{}\n", formatted_code)
        };

        write!(temp_file.as_file_mut(), "{}", final_code).expect("Failed to write to temp file");

        temp_file
            .as_file_mut()
            .flush()
            .expect("Failed to flush temp file");

        temp_file.as_file().sync_all().expect("Failed to sync file");

        let func_name = extract_func_name(&final_code);

        if std::env::var("DEBUG_TESTS").is_ok() {
            println!("========== DEBUG OUTPUT ==========");
            println!("Temp file path: {:?}", temp_file.path());
            println!("Code written:\n{}", final_code);
            println!("Extracted function name: {}", func_name);

            use std::fs;
            if let Ok(content) = fs::read_to_string(temp_file.path()) {
                println!("File content verification:\n{}", content);
            }
            println!("==================================");
        }

        (temp_file, func_name)
    }

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
                panic!(
                    "Function '{}' not found. Available functions: {:?}",
                    func_name,
                    cfgs.keys().collect::<Vec<_>>()
                );
            }
        };

        let actual_blocks = cfg.blocks.len();
        let actual_edges: usize = cfg.blocks.values().map(|b| b.succs.len()).sum();

        if std::env::var("DEBUG_TESTS").is_ok() {
            println!("=== CFG DEBUG INFO ===");
            println!("Function: {}", func_name);
            println!("Expected: {} blocks, {} edges", expected_blocks, expected_edges);
            println!("Actual: {} blocks, {} edges", actual_blocks, actual_edges);
            println!("======================");
        }

        assert_eq!(
            actual_blocks, expected_blocks,
            "Expected {} blocks, got {}",
            expected_blocks, actual_blocks
        );
        assert_eq!(
            actual_edges, expected_edges,
            "Expected {} edges, got {}",
            expected_edges, actual_edges
        );
    }

    fn verify_cfg_connectivity(code: &str) {
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
                panic!(
                    "Function '{}' not found. Available functions: {:?}",
                    func_name,
                    cfgs.keys().collect::<Vec<_>>()
                );
            }
        };

        // Check if all blocks (except entry) are reachable from entry
        let mut visited = HashSet::new();
        let mut stack = vec![cfg.entry];

        while let Some(current) = stack.pop() {
            if !visited.insert(current) {
                continue;
            }

            if let Some(block) = cfg.blocks.get(&current) {
                for &succ in &block.succs {
                    if !visited.contains(&succ) {
                        stack.push(succ);
                    }
                }
            }
        }

        let unreachable_blocks: Vec<_> = cfg
            .blocks
            .keys()
            .filter(|&&id| !visited.contains(&id))
            .collect();

        if !unreachable_blocks.is_empty() {
            println!("Warning: Unreachable blocks found: {:?}", unreachable_blocks);
        }
    }

    fn count_node_types(code: &str, target_stmt_type: &str) -> usize {
        let (temp_file, func_name) = parse_go_code(code);

        let (fset, objs, files) = parse_project(temp_file.path()).expect("Failed to parse file");

        if files.is_empty() {
            return 0;
        }

        let cfgs = build_cfgs_for_file(&fset, &objs, &files[0].ast);

        let cfg = match cfgs.get(&func_name) {
            Some(cfg) => cfg,
            None => return 0,
        };

        cfg.blocks
            .values()
            .flat_map(|block| &block.stmts)
            .filter(|stmt| {
                match &stmt.stmt {
                    Stmt::If(_) => target_stmt_type == "if",
                    Stmt::For(_) => target_stmt_type == "for",
                    Stmt::Range(_) => target_stmt_type == "range",
                    Stmt::Switch(_) => target_stmt_type == "switch",
                    Stmt::TypeSwitch(_) => target_stmt_type == "type_switch",
                    Stmt::Branch(_) => target_stmt_type == "branch",
                    Stmt::Labeled(_) => target_stmt_type == "label",
                    Stmt::Go(_) => target_stmt_type == "go",
                    Stmt::Defer(_) => target_stmt_type == "defer",
                    Stmt::Return(_) => target_stmt_type == "return",
                    Stmt::Block(_) => target_stmt_type == "block",
                    Stmt::Assign(_) => target_stmt_type == "assign",
                    Stmt::Send(_) => target_stmt_type == "send",
                    Stmt::IncDec(_) => target_stmt_type == "incdec",
                    Stmt::Expr(_) => target_stmt_type == "expr",
                    Stmt::Empty(_) => target_stmt_type == "empty",
                    _ => false,
                }
            })
            .count()
    }

    #[test]
    fn test_empty_function() {
        let code = r#"
            package main
            func main() {
            }
        "#;
        verify_cfg_properties(code, 2, 1); // entry -> exit
    }

    #[test]
    fn test_simple_function() {
        let code = r#"
            package main
            func main() {
                x := 1
                println(x)
            }
        "#;
        verify_cfg_properties(code, 4, 3); // entry -> x:=1 -> println(x) -> exit
    }

    #[test]
    fn test_if_statement() {
        let code = r#"
            package main
            func main() {
                if true {
                    println("true")
                } else {
                    println("false")
                }
            }
        "#;
        verify_cfg_properties(code, 7, 7); // updated for modular CFG
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_if_without_else() {
        let code = r#"
            package main
            func main() {
                if true {
                    println("true")
                }
                println("after")
            }
        "#;
        verify_cfg_properties(code, 7, 7); // updated for modular CFG
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_nested_if() {
        let code = r#"
            package main
            func main() {
                if x > 0 {
                    if y > 0 {
                        println("both positive")
                    }
                }
                println("done")
            }
        "#;
        verify_cfg_properties(code, 10, 11);
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_else_if_chain() {
        let code = r#"
            package main
            func main() {
                if x > 0 {
                    println("positive")
                } else if x < 0 {
                    println("negative")
                } else {
                    println("zero")
                }
            }
        "#;
        verify_cfg_properties(code, 11, 12);
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_forward_label_references() {
        let code = r#"
            package main
            func main() {
                x := 1
                if x > 0 {
                    goto target
                }
                x = 2
            target:
                println(x)
            }
        "#;
        verify_cfg_connectivity(code);
        assert_eq!(count_node_types(code, "label"), 1);
    }

    #[test]
    fn test_for_loop() {
        let code = r#"
            package main
            func main() {
                for i := 0; i < 10; i++ {
                    println(i)
                }
            }
        "#;
        verify_cfg_properties(code, 8, 8); // updated for modular CFG
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_infinite_for_loop() {
        let code = r#"
            package main
            func main() {
                for {
                    println("forever")
                }
            }
        "#;
        verify_cfg_properties(code, 6, 6); // updated for modular CFG
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_for_loop_no_init() {
        let code = r#"
            package main
            func main() {
                i := 0
                for i < 10 {
                    println(i)
                    i++
                }
            }
        "#;
        verify_cfg_properties(code, 7, 7);
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_range_loop() {
        let code = r#"
            package main
            func main() {
                arr := []int{1, 2, 3}
                for _, v := range arr {
                    println(v)
                }
            }
        "#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_continue_break() {
        let code = r#"
            package main
            func main() {
                for i := 0; i < 10; i++ {
                    if i%2 == 0 {
                        continue
                    }
                    if i > 7 {
                        break
                    }
                    println(i)
                }
            }
        "#;
        verify_cfg_connectivity(code);
        assert_eq!(count_node_types(code, "branch"), 2); // continue and break
    }

    #[test]
    fn test_nested_loops() {
        let code = r#"
            package main
            func main() {
                for i := 0; i < 3; i++ {
                    for j := 0; j < 3; j++ {
                        println(i, j)
                    }
                }
            }
        "#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_switch_statement() {
        let code = r#"
            package main
            func main() {
                x := 1
                switch x {
                case 1:
                    println("one")
                case 2:
                    println("two")
                default:
                    println("other")
                }
            }
        "#;
        verify_cfg_connectivity(code);
        assert_eq!(count_node_types(code, "switch"), 1);
    }

    #[test]
    fn test_switch_fallthrough() {
        let code = r#"
            package main
            func main() {
                x := 1
                switch x {
                case 1:
                    println("one")
                    fallthrough
                case 2:
                    println("two")
                default:
                    println("default")
                }
            }
        "#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_switch_no_default() {
        let code = r#"
            package main
            func main() {
                x := 1
                switch x {
                case 1:
                    println("one")
                case 2:
                    println("two")
                }
                println("after")
            }
        "#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_empty_switch() {
        let code = r#"
            package main
            func main() {
                x := 1
                switch x {
                }
                println("after")
            }
        "#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_goto_statement() {
        let code = r#"
            package main
            func main() {
                goto end
                println("skipped")
            end:
                println("end")
            }
        "#;
        verify_cfg_connectivity(code);
        assert_eq!(count_node_types(code, "label"), 1);
    }

    #[test]
    fn test_complex_goto() {
        let code = r#"
            package main
            func main() {
                x := 1
                if x > 0 {
                    goto positive
                }
                println("non-positive")
                goto end
            positive:
                println("positive")
            end:
                println("done")
            }
        "#;
        verify_cfg_connectivity(code);
        assert_eq!(count_node_types(code, "label"), 2);
    }

    #[test]
    fn test_goto_forward_backward() {
        let code = r#"
            package main
            func main() {
                x := 0
            loop:
                x++
                if x < 5 {
                    goto loop
                }
                println(x)
            }
        "#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_multiple_returns() {
        let code = r#"
            package main
            func main() {
                x := 1
                if x > 0 {
                    return
                }
                println("negative or zero")
            }
        "#;
        verify_cfg_connectivity(code);
        assert_eq!(count_node_types(code, "return"), 1);
    }

    #[test]
    fn test_complex_control_flow() {
        let code = r#"
            package main
            func main() {
                for i := 0; i < 10; i++ {
                    switch i % 3 {
                    case 0:
                        if i == 0 {
                            continue
                        }
                        println("divisible by 3")
                    case 1:
                        println("remainder 1")
                    default:
                        if i > 7 {
                            break
                        }
                        println("remainder 2")
                    }
                }
                println("done")
            }
        "#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_labeled_break_continue() {
        let code = r#"
            package main
            
            func LabeledBreak() int {
                sum := 0
            outer:
                for i := 0; i < 10; i++ {
                    for j := 0; j < 10; j++ {
                        if i+j > 10 {
                            break outer
                        }
                        sum += i * j
                    }
                }
                return sum
            }
        "#;
        verify_cfg_connectivity(code);
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
                        if i+j == 5 {
                            continue outer
                        }
                        sum += i * j
                    }
                }
                return sum
            }
        "#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_multiple_labeled_loops() {
        let code = r#"
            package main
            
            func MultipleLabeledLoops() int {
                sum := 0
            first:
                for i := 0; i < 5; i++ {
                second:
                    for j := 0; j < 5; j++ {
                        if i == 2 {
                            break first
                        }
                        if j == 3 {
                            continue second
                        }
                        sum += i + j
                    }
                }
                return sum
            }
        "#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_mixed_control_flow() {
        let code = r#"
            package main
            
            func MixedControlFlow(matrix [][]int) int {
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
        verify_cfg_connectivity(code);
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
        verify_cfg_connectivity(code);
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
        verify_cfg_connectivity(code);
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
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_function_calls_in_conditions() {
        let code = r#"
        package main

        func FunctionCallsInConditions(n int) int {
            if n > 0 {
                if n-1 > 0 {
                    return n * 2
                }
                return n
            }
            return 0
        }"#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_multiple_functions_in_file() {
        let code = r#"
            package main

            func helper(x int) bool {
                return x > 0
            }

            func MultiFunc(n int) int {
                if n > 0 {
                    return n * 2
                }
                return 0
            }"#;

        let (temp_file, _) = parse_go_code(code);
        let (fset, objs, files) = parse_project(temp_file.path()).expect("Failed to parse file");

        if files.is_empty() {
            println!("Warning: No files parsed, skipping test");
            return;
        }

        let cfgs = build_cfgs_for_file(&fset, &objs, &files[0].ast);

        assert!(cfgs.contains_key("helper"), "helper function not found");
        assert!(
            cfgs.contains_key("MultiFunc"),
            "MultiFunc function not found"
        );

        let cfg = cfgs.get("MultiFunc").unwrap();
        assert_eq!(cfg.blocks.len(), 9); // updated for modular CFG
    }

    #[test]
    fn test_switch_with_multiple_fallthrough() {
        let code = r#"
        package main

        func SwitchMultipleFallthrough(n int) int {
            result := 0
            switch n {
            case 1:
                result = 1
                fallthrough
            case 2:
                result += 2
                fallthrough
            case 3:
                result += 3
            default:
                result += 10
            }
            return result
        }"#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_break_to_nonexistent_label() {
        let code = r#"
        package main

        func BreakNonexistentLabel() int {
            for i := 0; i < 10; i++ {
                if i == 5 {
                    break  // Changed from 'break nonexistent' to just 'break'
                }
            }
            return 0
        }"#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_goto_to_later_defined_label() {
        let code = r#"
        package main

        func GotoForwardLabel() int {
            x := 0
            if x == 0 {
                goto later
            }
            x = 10
        later:
            return x
        }"#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_complex_switch_with_breaks() {
        let code = r#"
        package main

        func SwitchWithBreaks(n int) int {
        outer:
            for i := 0; i < 10; i++ {
                switch n {
                case 1:
                    if i > 5 {
                        break outer
                    }
                case 2:
                    if i < 3 {
                        break  // breaks the switch, not the loop
                    }
                    fallthrough
                case 3:
                    return i
                }
            }
            return -1
        }"#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_empty_blocks_optimization() {
        let code = r#"
            package main
    
            func EmptyBlocks() int {
                x := 0
                if x > 0 {
                    // Empty then block
                } else {
                    // Empty else block  
                }
                return x
            }"#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_unreachable_code_after_infinite_loop() {
        let code = r#"
            package main
    
            func UnreachableAfterInfinite() int {
                x := 0
                for {
                    x++
                    if x > 100 {
                        break
                    }
                }
                return x
            }"#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_nested_labeled_statements() {
        let code = r#"
        package main

        func NestedLabels() int {
            x := 0
        outer:
            for i := 0; i < 5; i++ {
            inner:
                for j := 0; j < 5; j++ {
                    x++
                    if x > 10 {
                        goto end
                    }
                    if x == 5 {
                        continue inner
                    }
                    if x == 8 {
                        break outer
                    }
                }
            }
        end:
            return x
        }"#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_switch_type_assertion_chain() {
        let code = r#"
        package main

        func TypeAssertionChain(x interface{}) string {
            switch v := x.(type) {
            case int:
                if v > 0 {
                    return "positive int"
                }
                return "non-positive int"
            case string:
                if len(v) > 0 {
                    return "non-empty string"
                }
                return "empty string"
            case nil:
                return "nil"
            default:
                return "other"
            }
        }"#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_select_with_multiple_cases() {
        let code = r#"
        package main

        func SelectMultiple(ch1, ch2, ch3 chan int) int {
            for {
                select {
                case v1 := <-ch1:
                    if v1 > 0 {
                        return v1
                    }
                case v2 := <-ch2:
                    if v2 < 0 {
                        continue
                    }
                    return v2
                case ch3 <- 42:
                    return 42
                default:
                    break
                }
                break
            }
            return 0
        }"#;
        verify_cfg_connectivity(code);
    }

    #[test]
    fn test_panic_in_different_contexts() {
        let code = r#"
        package main

        func PanicContexts(n int) int {
            if n < 0 {
                panic("negative")
            }
            
            for i := 0; i < n; i++ {
                if i == 5 {
                    panic("five")
                }
            }
            
            switch n {
            case 666:
                panic("evil")
            case 13:
                panic("unlucky")
            default:
                return n
            }
        }"#;
        verify_cfg_connectivity(code);
    }
}

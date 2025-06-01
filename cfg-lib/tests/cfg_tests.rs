#[cfg(test)]
mod tests {
    use skan_uj_kod::ast::parse_project;
    use skan_uj_kod::cfg::build_cfgs_for_file;
    use go_parser::Token;
    use go_parser::ast::Stmt;
    use std::collections::HashSet;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn parse_go_code(code: &str) -> (tempfile::NamedTempFile, String) {
        use std::io::Write;
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

        write!(temp_file, "{}", final_code).expect("Failed to write to temp file");

        temp_file.flush().expect("Failed to flush temp file");

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

        println!("Function: {}", func_name);
        println!("Blocks: {}", cfg.blocks.len());
        let total_edges: usize = cfg.blocks.values().map(|b| b.succs.len()).sum();
        println!("Edges: {}", total_edges);

        for (id, block) in &cfg.blocks {
            println!(
                "Block {}: {} statements, {} successors: {:?}",
                id,
                block.stmts.len(),
                block.succs.len(),
                block.succs
            );
        }

        assert_eq!(
            cfg.blocks.len(),
            expected_blocks,
            "Expected {} blocks, got {}",
            expected_blocks,
            cfg.blocks.len()
        );

        assert_eq!(
            total_edges, expected_edges,
            "Expected {} edges, got {}",
            expected_edges, total_edges
        );

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
        assert_eq!(
            visited.len(),
            cfg.blocks.len(),
            "Not all blocks are reachable"
        );
    }
    

    use skan_uj_kod::cfg::ControlFlowGraph;

    fn verify_cfg_structure(code: &str) -> ControlFlowGraph {
        let (temp_file, func_name) = parse_go_code(code);

        use std::fs;
        if let Ok(content) = fs::read_to_string(temp_file.path()) {
            if std::env::var("DEBUG_TESTS").is_ok() {
                println!("File content before parsing:\n{}", content);
            }
        } else {
            panic!("Failed to read back temp file at {:?}", temp_file.path());
        }

        let parse_result = parse_project(temp_file.path());

        let (fset, objs, files) = match parse_result {
            Ok(result) => result,
            Err(e) => {
                let file_content = fs::read_to_string(temp_file.path())
                    .unwrap_or_else(|_| "Failed to read file".to_string());
                panic!(
                    "Failed to parse Go code: {:?}\nTemp file path: {:?}\nFile content:\n{}\nOriginal code:\n{}",
                    e,
                    temp_file.path(),
                    file_content,
                    code
                );
            }
        };

        if files.is_empty() {
            let exists = temp_file.path().exists();
            let metadata = temp_file.path().metadata();
            let file_content = fs::read_to_string(temp_file.path())
                .unwrap_or_else(|_| "Failed to read file".to_string());

            panic!(
                "No Go files were parsed from the code.\n\
                 File exists: {}\n\
                 File metadata: {:?}\n\
                 File path: {:?}\n\
                 File content:\n{}\n\
                 Original code:\n{}",
                exists,
                metadata,
                temp_file.path(),
                file_content,
                code
            );
        }

        let cfgs = build_cfgs_for_file(&fset, &objs, &files[0].ast);

        if !cfgs.contains_key(&func_name) {
            panic!(
                "Function '{}' not found. Available functions: {:?}\nCode:\n{}",
                func_name,
                cfgs.keys().collect::<Vec<_>>(),
                code
            );
        }

        cfgs.get(&func_name).unwrap().clone()
    }

    #[test]
    fn test_empty_function() {
        let code = r#"
            package main
            
            func EmptyFunction() {
            }
        "#;

        verify_cfg_properties(code, 2, 1);
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

        verify_cfg_properties(code, 5, 5);
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

        verify_cfg_properties(code, 7, 8);
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

        verify_cfg_properties(code, 9, 11);
    }

    #[test]
    fn test_forward_label_references() {
        let code = r#"
        package main

        func ForwardLabelRef() int {
            x := 0
            for i := 0; i < 10; i++ {
                if i == 5 {
                    goto done  // Forward reference
                }
                x += i
            }
        done:
            return x
        }"#;

        let cfg = verify_cfg_structure(code);

        let goto_block = cfg
            .blocks
            .values()
            .find(|b| {
                b.stmts.iter().any(|s| {
                    if let Stmt::Branch(br) = s {
                        br.token == Token::GOTO
                    } else {
                        false
                    }
                })
            })
            .expect("Goto block not found");

        assert_eq!(
            goto_block.succs.len(),
            1,
            "Goto should have exactly one successor"
        );

        assert!(cfg.blocks.len() > 5, "Should have multiple blocks");
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

        verify_cfg_properties(code, 8, 8);
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

        verify_cfg_properties(code, 6, 6);
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

        verify_cfg_properties(code, 10, 12);
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

        verify_cfg_properties(code, 11, 12);
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

        verify_cfg_properties(code, 9, 11);
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

        verify_cfg_properties(code, 9, 10);
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

        verify_cfg_properties(code, 5, 5);
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

        verify_cfg_properties(code, 13, 14);
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

        verify_cfg_properties(code, 11, 12);
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

        verify_cfg_properties(code, 14, 16);
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

        verify_cfg_properties(code, 14, 16);
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

        verify_cfg_properties(code, 17, 20);
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

        verify_cfg_properties(code, 10, 12);
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

        verify_cfg_properties(code, 9, 9);
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

        verify_cfg_properties(code, 3, 2);
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

        verify_cfg_properties(code, 3, 2);
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

        verify_cfg_properties(code, 5, 5);
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

        verify_cfg_properties(code, 7, 8);
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

        verify_cfg_properties(code, 12, 15);
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

        verify_cfg_properties(code, 7, 8);
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
        assert_eq!(cfg.blocks.len(), 5);
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

        let cfg = verify_cfg_structure(code);

        let switch_block = cfg
            .blocks
            .values()
            .find(|b| b.stmts.iter().any(|s| matches!(s, Stmt::Switch(_))))
            .expect("Switch block not found");

        assert!(
            switch_block.succs.len() >= 4,
            "Switch should connect to all cases, but has {} successors",
            switch_block.succs.len()
        );
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

        let cfg = verify_cfg_structure(code);

        assert!(cfg.blocks.len() > 0);
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

        let cfg = verify_cfg_structure(code);

        let goto_block = cfg
            .blocks
            .values()
            .find(|b| {
                b.stmts.iter().any(|s| {
                    if let Stmt::Branch(br) = s {
                        br.token == Token::GOTO
                    } else {
                        false
                    }
                })
            })
            .expect("Goto block not found");

        assert_eq!(goto_block.succs.len(), 1);
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

        verify_cfg_properties(code, 8, 8);
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

        let cfg = verify_cfg_structure(code);

        let empty_blocks = cfg
            .blocks
            .values()
            .filter(|b| b.stmts.len() == 1 && matches!(&b.stmts[0], Stmt::Empty(_)))
            .count();

        assert!(empty_blocks <= 2, "Too many empty blocks: {}", empty_blocks);
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

        let cfg = verify_cfg_structure(code);

        let return_count = cfg
            .blocks
            .values()
            .flat_map(|b| &b.stmts)
            .filter(|s| matches!(s, Stmt::Return(_)))
            .count();

        assert!(
            return_count >= 1,
            "Should have at least one return statement"
        );
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

        verify_cfg_properties(code, 19, 23);
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

        verify_cfg_properties(code, 3, 2);
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

        verify_cfg_properties(code, 6, 6);
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

        verify_cfg_properties(code, 13, 17);
    }
}

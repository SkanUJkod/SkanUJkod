#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{self, Read, Write};
    use parser_lib::parser_manager::{ParserManager, ParserTrait};
    use parser_lib::language_parsers::go_parser::GoParser;
    use parser_lib::ast_node::AstNode;
    use std::str;

    fn read_file(file_path: &str) -> io::Result<String> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    #[test]
    fn test_parse_go_file() {
        let file_path = "../test_go_project/simple.go";
        let source_code = match read_file(file_path) {
            Ok(code) => code,
            Err(e) => panic!("Failed to read file: {}", e),
        };

        let mut parser_manager = ParserManager::new();
        let go_parser = GoParser::new();
        parser_manager.add_parser("go".to_string(), go_parser);

        let result = parser_manager.parse("go", file_path, &source_code);

        let mut output = Vec::new();
        match result {
            Ok(ast) => {
                writeln!(output, "Parsing successful!").unwrap();
                let buffer: &mut dyn Write = &mut output;
                ast.write_tree(buffer).unwrap();
            },
            Err(e) => {
                panic!("Parsing failed: {}", e);
            }
        }

        let output_str = str::from_utf8(&output).expect("Failed to convert bytes to string");
        let expected_output = "Parsing successful!\nFile\n  Package\n  Import\n  Generic\n  Function\n    Signature\n    Body\n";
        assert_eq!(output_str, expected_output);
    }

    mod ast_node_tests {
        use super::*;

        fn create_test_tree() -> AstNode {
            let mut root = AstNode::new("Root");
            
            let mut child1 = AstNode::new("Child1");
            let grandchild1 = AstNode::new("GrandChild1");
            let grandchild2 = AstNode::new("GrandChild2");
            
            child1.add_child(grandchild1);
            child1.add_child(grandchild2);
            
            let child2 = AstNode::new("Child2");
            
            root.add_child(child1);
            root.add_child(child2);
            
            root
        }

        #[test]
        fn test_ast_node_new() {
            let node = AstNode::new("TestNode");
            assert_eq!(node.kind, "TestNode");
            assert_eq!(node.child_count(), 0);
        }

        #[test]
        fn test_add_child() {
            let mut parent = AstNode::new("Parent");
            let child = AstNode::new("Child");
            parent.add_child(child);
            
            assert_eq!(parent.child_count(), 1);
            assert_eq!(parent.children[0].kind, "Child");
        }

        #[test]
        fn test_recursive_count() {
            let ast = create_test_tree();
            assert_eq!(ast.recursive_count(), 5);
        }

        #[test]
        fn test_pre_order_traversal() {
            let ast = create_test_tree();
            let mut visited_nodes = Vec::new();
            
            ast.pre_order_traversal(&mut |node| {
                visited_nodes.push(node.kind.clone());
            });
            
            let expected = vec!["Root", "Child1", "GrandChild1", "GrandChild2", "Child2"];
            assert_eq!(visited_nodes, expected);
        }

        #[test]
        fn test_post_order_traversal() {
            let ast = create_test_tree();
            let mut visited_nodes = Vec::new();
            
            ast.post_order_traversal(&mut |node| {
                visited_nodes.push(node.kind.clone());
            });
            
            let expected = vec!["GrandChild1", "GrandChild2", "Child1", "Child2", "Root"];
            assert_eq!(visited_nodes, expected);
        }

        #[test]
        fn test_breadth_first_traversal() {
            let ast = create_test_tree();
            let mut visited_nodes = Vec::new();
            
            ast.breadth_first_traversal(|node| {
                visited_nodes.push(node.kind.clone());
            });
            
            let expected = vec!["Root", "Child1", "Child2", "GrandChild1", "GrandChild2"];
            assert_eq!(visited_nodes, expected);
        }

        #[test]
        fn test_find_nodes() {
            let ast = create_test_tree();
            
            let nodes = ast.find_nodes(|node| node.kind.contains("Child"));
            
            assert_eq!(nodes.len(), 4);
            
            let node_kinds: Vec<String> = nodes.iter().map(|n| n.kind.clone()).collect();
            assert!(node_kinds.contains(&"Child1".to_string()));
            assert!(node_kinds.contains(&"Child2".to_string()));
            assert!(node_kinds.contains(&"GrandChild1".to_string()));
            assert!(node_kinds.contains(&"GrandChild2".to_string()));
        }

        #[test]
        fn test_find_nodes_by_kind() {
            let ast = create_test_tree();
            
            let nodes = ast.find_nodes_by_kind("GrandChild1");
            
            assert_eq!(nodes.len(), 1);
            assert_eq!(nodes[0].kind, "GrandChild1");
        }

        #[test]
        fn test_write_tree() {
            let ast = create_test_tree();
            let mut output = Vec::new();
            
            ast.write_tree(&mut output).unwrap();
            
            let output_str = str::from_utf8(&output).expect("Failed to convert bytes to string");
            let expected_output = "Root\n  Child1\n    GrandChild1\n    GrandChild2\n  Child2\n";
            
            assert_eq!(output_str, expected_output);
        }
        
        #[test]
        fn test_empty_tree() {
            let node = AstNode::new("Empty");
            assert_eq!(node.recursive_count(), 1);
            
            let mut output = Vec::new();
            node.write_tree(&mut output).unwrap();
            let output_str = str::from_utf8(&output).expect("Failed to convert bytes to string");
            assert_eq!(output_str, "Empty\n");
            
            let nodes = node.find_nodes(|_| true);
            assert_eq!(nodes.len(), 1);
        }
        
        #[test]
        fn test_large_tree() {
            let mut root = AstNode::new("Root");
            let mut count = 1;
            
            for i in 0..10 {
                let mut level1 = AstNode::new(format!("Level1_{}", i));
                for j in 0..5 {
                    let mut level2 = AstNode::new(format!("Level2_{}_{}", i, j));
                    for k in 0..3 {
                        let level3 = AstNode::new(format!("Level3_{}_{}_{}", i, j, k));
                        level2.add_child(level3);
                        count += 1;
                    }
                    level1.add_child(level2);
                    count += 1;
                }
                root.add_child(level1);
                count += 1;
            }
            
            assert_eq!(root.recursive_count(), count);
            
            let level3_nodes = root.find_nodes(|node| node.kind.starts_with("Level3"));
            assert_eq!(level3_nodes.len(), 10 * 5 * 3);
        }
        
        #[test]
        fn test_serialize_to_json() {
            let ast = create_test_tree();
            let mut json_str = String::new();
            
            serialize_to_json(&ast, &mut json_str);
            
            assert!(json_str.contains("\"kind\":\"Root\""));
            assert!(json_str.contains("\"children\":["));
            assert!(json_str.contains("\"kind\":\"Child1\""));
            assert!(json_str.contains("\"kind\":\"GrandChild1\""));
            assert!(json_str.contains("\"kind\":\"GrandChild2\""));
            assert!(json_str.contains("\"kind\":\"Child2\""));
        }
        
        fn serialize_to_json(node: &AstNode, output: &mut String) {
            output.push_str(&format!("{{\"kind\":\"{}\",\"children\":[", node.kind));
            
            for (i, child) in node.children.iter().enumerate() {
                if i > 0 {
                    output.push_str(",");
                }
                serialize_to_json(child, output);
            }
            
            output.push_str("]}");
        }
        
        #[test]
        fn test_serialize_to_xml() {
            let ast = create_test_tree();
            let mut xml_str = String::new();
            
            serialize_to_xml(&ast, &mut xml_str, 0);
            
            assert!(xml_str.contains("<node kind=\"Root\">"));
            assert!(xml_str.contains("<node kind=\"Child1\">"));
            assert!(xml_str.contains("<node kind=\"GrandChild1\">"));
            assert!(xml_str.contains("<node kind=\"GrandChild2\">"));
            assert!(xml_str.contains("<node kind=\"Child2\">"));
            assert!(xml_str.contains("</node>"));
        }
        
        fn serialize_to_xml(node: &AstNode, output: &mut String, indent: usize) {
            let indent_str = " ".repeat(indent);
            output.push_str(&format!("{}<node kind=\"{}\">\n", indent_str, node.kind));
            
            for child in &node.children {
                serialize_to_xml(child, output, indent + 2);
            }
            
            output.push_str(&format!("{}</node>\n", indent_str));
        }
    }

    mod parser_manager_tests {
        use super::*;
        use std::cell::RefCell;

        struct MockParser {
            should_fail: bool,
            errors: RefCell<Vec<String>>,
        }

        impl MockParser {
            fn new(should_fail: bool) -> Self {
                MockParser {
                    should_fail,
                    errors: RefCell::new(Vec::new()),
                }
            }
        }

        impl ParserTrait for MockParser {
            fn parse(&mut self, _file_path: &str, _source_code: &str) -> Result<AstNode, String> {
                if self.should_fail {
                    self.errors.borrow_mut().push("Mock parsing error".to_string());
                    Err("Mock parsing failed".to_string())
                } else {
                    let mut root = AstNode::new("MockRoot");
                    root.add_child(AstNode::new("MockChild"));
                    Ok(root)
                }
            }

            fn get_errors(&self) -> Vec<String> {
                self.errors.borrow().clone()
            }
        }

        #[test]
        fn test_parser_manager_new() {
            let _parser_manager: ParserManager<MockParser> = ParserManager::new();
            assert!(true);
        }

        #[test]
        fn test_add_and_get_parser() {
            let mut parser_manager = ParserManager::new();
            let mock_parser = MockParser::new(false);
            
            parser_manager.add_parser("mock".to_string(), mock_parser);
            
            let parser = parser_manager.get_parser("mock");
            assert!(parser.is_some());
            
            let non_existent = parser_manager.get_parser("nonexistent");
            assert!(non_existent.is_none());
        }

        #[test]
        fn test_parse_success() {
            let mut parser_manager = ParserManager::new();
            let mock_parser = MockParser::new(false);
            
            parser_manager.add_parser("mock".to_string(), mock_parser);
            
            let result = parser_manager.parse("mock", "dummy.txt", "dummy code");
            
            assert!(result.is_ok());
            let ast = result.unwrap();
            assert_eq!(ast.kind, "MockRoot");
            assert_eq!(ast.child_count(), 1);
            assert_eq!(ast.children[0].kind, "MockChild");
        }

        #[test]
        fn test_parse_failure() {
            let mut parser_manager = ParserManager::new();
            let mock_parser = MockParser::new(true);
            
            parser_manager.add_parser("mock".to_string(), mock_parser);
            
            let result = parser_manager.parse("mock", "dummy.txt", "dummy code");
            
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.contains("Failed to parse mock code"));
            assert!(error.contains("Mock parsing error"));
        }

        #[test]
        fn test_parse_unknown_language() {
            let mut parser_manager: ParserManager<MockParser> = ParserManager::new();
            
            let result = parser_manager.parse("unknown", "dummy.txt", "dummy code");
            
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error, "No parser available for language: unknown");
        }
    }

    mod go_parser_tests {
        use super::*;
        use parser_lib::language_parsers::go_parser::GoParser;

        #[test]
        fn test_go_parser_initialization() {
            let _parser = GoParser::new();
            assert!(true);
        }

        #[test]
        fn test_parse_valid_go_code() {
            let mut parser = GoParser::new();
            let code = "package main\n\nfunc main() {\n\tfmt.Println(\"Hello\")\n}";
            
            let result = parser.parse("test.go", code);
            
            assert!(result.is_ok());
            
            let ast = result.unwrap();
            assert_eq!(ast.kind, "File");
            
            let function_nodes = ast.find_nodes_by_kind("Function");
            assert!(!function_nodes.is_empty());
        }

        #[test]
        fn test_export_ast_structure() {
            let mut parser = GoParser::new();
            let code = "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"Hello\")\n}";
            
            let result = parser.parse("test.go", code);
            assert!(result.is_ok());
            
            let ast = result.unwrap();
            
            let mut output = Vec::new();
            ast.write_tree(&mut output).unwrap();
            let output_str = str::from_utf8(&output).expect("Failed to convert bytes to string");
            
            assert!(output_str.contains("Package"));
            assert!(output_str.contains("Import"));
            assert!(output_str.contains("Function"));
            
            assert!(output_str.contains("Signature"));
            assert!(output_str.contains("Body"));
        }
        
        #[test]
        fn test_parse_invalid_go_code() {
            let mut parser = GoParser::new();
            let invalid_code = "package main\n\nfunc main() {\n\tfmt.Println(\"Hello\"\n}";
            
            let result = parser.parse("test.go", invalid_code);
            
            assert!(result.is_err());
        }
        
        #[test]
        fn test_get_errors() {
            let mut parser = GoParser::new();
            let invalid_code = "package main\n\nfunc main() {\n\tfmt.Println(\"Hello\"\n}";
            
            let _ = parser.parse("test.go", invalid_code);
            
            let errors = parser.get_errors();
            assert!(!errors.is_empty());
            
            for error in errors {
                assert!(error.contains("Error at"));
                assert!(error.contains("test.go"));
            }
        }
        
        #[test]
        fn test_convert_node_hierarchy() {
            let mut parser = GoParser::new();
            let code = "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"Hello\")\n\tfmt.Println(\"World\")\n}";
            
            let result = parser.parse("test.go", code);
            assert!(result.is_ok());
            
            let ast = result.unwrap();
            
            assert!(ast.find_nodes_by_kind("File").len() == 1);
            assert!(ast.find_nodes_by_kind("Package").len() == 1);
            assert!(ast.find_nodes_by_kind("Import").len() == 1);
            assert!(ast.find_nodes_by_kind("Function").len() == 1);
            
            let function_nodes = ast.find_nodes_by_kind("Function");
            let function = &function_nodes[0];
            
            assert_eq!(function.child_count(), 2);
            assert_eq!(function.children[0].kind, "Signature");
            assert_eq!(function.children[1].kind, "Body");
        }
        
        #[test]
        fn test_multiple_parse_calls() {
            let mut parser = GoParser::new();
            
            let code1 = "package main\n\nfunc foo() {}\n";
            let result1 = parser.parse("test1.go", code1);
            assert!(result1.is_ok());
            
            let code2 = "package main\n\nfunc bar() {}\n";
            let result2 = parser.parse("test2.go", code2);
            assert!(result2.is_ok());
            
            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();
            
            assert_eq!(ast1.kind, "File");
            assert_eq!(ast2.kind, "File");
            
            let functions1 = ast1.find_nodes_by_kind("Function");
            let functions2 = ast2.find_nodes_by_kind("Function");
            
            assert_eq!(functions1.len(), 1);
            assert_eq!(functions2.len(), 1);
        }
    }
}
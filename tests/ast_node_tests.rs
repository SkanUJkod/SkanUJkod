#[cfg(test)]
mod tests {
    use std::str;
    use skan_uj_kod::parser::{ CodeParser };
    use skan_uj_kod::ast_node::{ AstNode };
    use tree_sitter::Tree;

    fn parse_source(source_code: &str) -> Tree {
        let mut parser = CodeParser::with_default_language()
            .expect("Failed to initialize parser with default language");
        parser.parse(source_code).expect("Parsing failed")
    }
    
    #[test]
    fn test_pre_order_traversal() {
        let source_code = r#"
            package main
            import "fmt"
            func main() {
                var x = 3 + 4
            }
        "#;
        let tree = parse_source(source_code);
        let root_node = tree.root_node();
        let ast_node = AstNode::new(root_node);

        let mut nodes_visited = Vec::new();
        ast_node.pre_order_traversal(&mut |node| {
            nodes_visited.push(node.kind().to_string());
            println!("Visited node: {}", node.kind());
        });

        println!("Visited nodes: {:?}", nodes_visited);
        assert!(nodes_visited.contains(&"source_file".to_string()),"Node \"source_file\" should be present");
        assert!(nodes_visited.contains(&"function_declaration".to_string()), "Function declaration should be present");
        assert!(nodes_visited.contains(&"var_declaration".to_string()), "Variable declaration should be present");
        assert!(nodes_visited.contains(&"binary_expression".to_string()), "Binary expression should be present");

    }

    #[test]
    fn test_post_order_traversal() {
        let source_code = r#"
            package main
            import "fmt"
            func main() {
                var x = 3 + 4
            }
        "#;
        let tree = parse_source(source_code);
        let root_node = tree.root_node();
        let ast_node = AstNode::new(root_node);

        let mut nodes_visited = Vec::new();
        ast_node.post_order_traversal(&mut |node| {
            nodes_visited.push(node.kind().to_string());
        });

        println!("Visited nodes: {:?}", nodes_visited);
        assert!(nodes_visited.last().unwrap() == &"source_file".to_string());
    }

    #[test]
    fn test_breadth_first_traversal() {
        let source_code = r#"
            package main
            import "fmt"
            func main() {
                var x = 3 + 4
            }
        "#;
        let tree = parse_source(source_code);
        let root_node = tree.root_node();
        let ast_node = AstNode::new(root_node);

        let mut nodes_visited = Vec::new();
        ast_node.breadth_first_traversal(|node| {
            nodes_visited.push(node.kind().to_string());
        });

        println!("Visited nodes: {:?}", nodes_visited);
        assert!(nodes_visited.contains(&"source_file".to_string()));
        assert!(nodes_visited.iter().filter(|&k| k == "function_declaration").count() == 1);
    }

    #[test]
    fn test_find_nodes_by_type() {
        let source_code = r#"
            package main
            import "fmt"
            func main() {
                var x = 3 + 4
            }
        "#;
        let tree = parse_source(source_code);
        let root_node = tree.root_node();
        let ast_node = AstNode::new(root_node);

        let function_nodes = ast_node.find_nodes_by_type("function_declaration");
        let var_declarations = ast_node.find_nodes_by_type("var_declaration");
        assert_eq!(function_nodes.len(), 1, "Expected one function declaration node.");
        assert_eq!(var_declarations.len(), 1, "Expected one let statement node.");
    }

    fn normalize_whitespace(s: &str) -> String {
        s.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    #[test]
    fn test_print_tree() {
        let source_code = r#"
            package main

            import "fmt"

            func main() {
                var x = 3 + 4
                fmt.Println(x)
            }
        "#;
        
        let tree = parse_source(source_code);
        let root_node = tree.root_node();
        let ast_root = AstNode::new(root_node);

        let mut buffer = Vec::new();
        ast_root.print_tree(&mut buffer).expect("Failed to write to buffer");

        let output = str::from_utf8(&buffer).expect("Failed to convert to utf8");
        println!("Raw output:\n{}", output);

        let expected_output = "\
source_file
  package_clause
    package
    package_identifier

  import_declaration
    import
    import_spec
      interpreted_string_literal
        \"
        \"

  function_declaration
    func
    identifier
    parameter_list
      (
      )
    block
      {
      var_declaration
        var
        var_spec
          identifier
          =
          expression_list
            binary_expression
              int_literal
              +
              int_literal

      expression_statement
        call_expression
          selector_expression
            identifier
            .
            field_identifier
          argument_list
            (
            identifier
            )

      }
";
        let normalized_output = normalize_whitespace(output);
        let normalized_expected = normalize_whitespace(expected_output);
        assert_eq!(normalized_output, normalized_expected, "The printed AST does not match the expected output.");
    }
}
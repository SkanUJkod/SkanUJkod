#[cfg(test)]
mod tests {
    use skan_uj_kod::parser::{create_language_registry, CodeParser};

    #[test]
    fn test_code_parser_new() {
        let registry = create_language_registry();
        let parser = CodeParser::new(&registry, "go");
        assert!(parser.is_some(), "Failed to initialize CodeParser with specified language.");
    }

    #[test]
    fn test_code_parser_with_default_language() {
        let _parser = CodeParser::with_default_language()
            .expect("Failed to initialize parser with default language");
        println!("Default parser for 'go' initialized successfully.");
    }

    #[test]
    fn test_parse() {
        let registry = create_language_registry();
        let mut parser = CodeParser::new(&registry, "go").expect("Parser initialization failed.");
        
        let code = r#"
            package main
            
            import "fmt"
            
            func main() {
                fmt.Println("Hello, World!")
            }
        "#;
        
        if let Some(tree) = parser.parse(code) {
            assert!(!tree.root_node().has_error(), "Parsing errors found in Go code.");
        } else {
            panic!("Failed to parse Go code.");
        }
    }
}
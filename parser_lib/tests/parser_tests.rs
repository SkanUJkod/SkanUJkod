#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{self, Read, Write};
    use parser_lib::parser_manager::ParserManager;
    use parser_lib::language_parsers::go_parser::GoParser;
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
}
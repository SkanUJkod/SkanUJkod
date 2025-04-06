use crate::ast_node::AstNode;
use std::collections::HashMap;

pub trait ParserTrait {
    fn parse(&mut self, file_path: &str, source_code: &str) -> Result<AstNode, String>;
    fn get_errors(&self) -> Vec<String>;
}

pub struct ParserManager<T: ParserTrait> {
    parsers: HashMap<String, T>,
}

impl<T: ParserTrait> ParserManager<T> {
    pub fn new() -> Self {
        ParserManager {
            parsers: HashMap::new(),
        }
    }

    pub fn add_parser(&mut self, language: String, parser: T) {
        self.parsers.insert(language, parser);
    }

    pub fn get_parser(&self, language: &str) -> Option<&T> {
        self.parsers.get(language)
    }

    pub fn parse(&mut self, language: &str, file_path: &str, source_code: &str) -> Result<AstNode, String> {
        if let Some(parser) = self.parsers.get_mut(language) {
            parser.parse(file_path, source_code).map_err(|err| {
                format!(
                    "Failed to parse {} code: {}\nErrors: {:?}",
                    language, err, parser.get_errors()
                )
            })
        } else {
            Err(format!("No parser available for language: {}", language))
        }
    }
}
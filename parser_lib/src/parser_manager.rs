use crate::ast_node::AstNode;
use std::collections::HashMap;

pub trait ParserTrait {
    fn parse(&mut self, file_path: &str, source_code: &str) -> Result<AstNode, String>;
    fn get_errors(&self) -> Vec<String>;
}

pub struct ParserManager<T: ParserTrait> {
    parsers: HashMap<String, T>,
}

impl<T: ParserTrait> Default for ParserManager<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ParserTrait> ParserManager<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
        }
    }

    #[must_use]
    pub fn add_parser(&mut self, language: String, parser: T) {
        self.parsers.insert(language, parser);
    }

    #[must_use]
    pub fn get_parser(&self, language: &str) -> Option<&T> {
        self.parsers.get(language)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn parse(
        &mut self,
        language: &str,
        file_path: &str,
        source_code: &str,
    ) -> Result<AstNode, String> {
        self.parsers.get_mut(language).map_or_else(
            || Err(format!("No parser available for language: {language}")),
            |parser| {
                parser.parse(file_path, source_code).map_err(|err| {
                    format!(
                        "Failed to parse {} code: {}\nErrors: {:?}",
                        language,
                        err,
                        parser.get_errors()
                    )
                })
            },
        )
    }
}

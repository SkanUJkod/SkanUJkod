extern crate tree_sitter;
extern crate tree_sitter_go;

use tree_sitter::{Parser, Language };
unsafe extern "C"  { fn tree_sitter_go() -> Language; }


fn get_go_language() -> tree_sitter::Language {
    unsafe { tree_sitter_go() }
}

struct LanguageInfo {
    name: &'static str,
    get_language_fn: unsafe fn() -> Language,
}

pub struct LanguageRegistry {
    languages: Vec<LanguageInfo>,
}

impl LanguageRegistry {
    pub fn new() -> Self {
        LanguageRegistry {
            languages: Vec::new(),
        }
    }

    pub fn register_language(&mut self, name: &'static str, get_language_fn: unsafe fn() -> Language) {
        self.languages.push(LanguageInfo { name, get_language_fn });
    }

    pub fn get_language(&self, name: &str) -> Option<Language> {
        self.languages.iter()
            .find(|lang| lang.name == name)
            .map(|lang_info| unsafe { (lang_info.get_language_fn)() })
    }   
}

pub fn create_language_registry() -> LanguageRegistry {
    let mut registry = LanguageRegistry::new();
    registry.register_language("go", get_go_language);
    registry
}

pub struct CodeParser {
    parser: Parser,
}

impl CodeParser {
    pub fn new(language_registry: &LanguageRegistry, language_name: &str) -> Option<Self> {
        let mut parser = Parser::new();
        if let Some(language) = language_registry.get_language(language_name) {
            parser.set_language(language).ok()?;
            Some(CodeParser { parser })
        } else {
            None // Language not found
        }
    }

    pub fn with_default_language() -> Option<Self> {
        let language_registry = create_language_registry();
        CodeParser::new(&language_registry, "go")
    }

    pub fn parse(&mut self, source_code: &str) -> Option<tree_sitter::Tree> {
        self.parser.parse(source_code, None)
    }
}

use abi_stable::std_types::RString;
use std::fmt::{self, Display};

#[derive(Debug)]
pub struct ParseFileResult {
    pub file: go_parser::ast::File,
    pub ast_objects: go_parser::AstObjects,
}

impl Display for ParseFileResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&0, f)
    }
}

pub struct ParseFileDep {}

pub struct ParseFileParams {
    pub filepath: RString,
}

pub fn parse_file(_deps: ParseFileDep, params: &ParseFileParams) -> ParseFileResult {
    let source =
        std::fs::read_to_string(params.filepath.as_str()).expect("Failed to open go source file");

    let mut fs = go_parser::FileSet::new();
    let mut o = go_parser::AstObjects::new();
    let el = &mut go_parser::ErrorList::new();

    let (_parser, maybe_file) =
        go_parser::parse_file(&mut o, &mut fs, el, params.filepath.as_str(), &source, true);

    ParseFileResult {
        file: maybe_file.expect("Something went wrong when trying to parse the source"),
        ast_objects: o,
    }
}

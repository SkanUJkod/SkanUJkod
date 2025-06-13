use abi_stable::std_types::RString;
use go_parser::{AstObjects, FileSet};
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(Debug)]
pub struct ParseDirResult {
    pub packages: HashMap<String, go_parser::ast::Package>,
    pub ast_objects: AstObjects,
    pub fileset: FileSet,
}

impl Display for ParseDirResult {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO
        Ok(())
    }
}

pub struct ParseDirDep {}

pub struct ParseDirParams {
    pub dir: RString,
}

pub fn parse_dir(_deps: ParseDirDep, params: &ParseDirParams) -> ParseDirResult {
    let mut fs = go_parser::FileSet::new();
    let mut o = go_parser::AstObjects::new();
    let el = &mut go_parser::ErrorList::new();

    ParseDirResult {
        packages: go_parser::parse_dir(&mut o, &mut fs, el, params.dir.as_str(), "", true, None)
            .expect("Parsing project failed"),
        ast_objects: o,
        fileset: fs,
    }
}

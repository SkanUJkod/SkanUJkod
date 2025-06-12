use abi_stable::std_types::RString;
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(Debug)]
pub struct ParseDirResult {
    pub result: HashMap<String, go_parser::ast::Package>,
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
        result: go_parser::parse_dir(&mut o, &mut fs, el, params.dir.as_str(), "", true, None)
            .expect("Parsing project failed"),
    }
}

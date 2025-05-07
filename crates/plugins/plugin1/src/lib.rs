use abi_stable::{rvec, std_types::RVec};
use plugin_interface::{BoxedPFResult, BoxedUserParam, Plugin, Plugin_Ref, PluginFunction};
use std::fmt::{self, Display};

use abi_stable::{
    DynTrait, export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn, std_types::RString,
};

#[export_root_module]
pub fn get_library() -> Plugin_Ref {
    Plugin { funcs: new_pf_vec }.leak_into_prefix()
}

#[sabi_extern_fn]
fn new_pf_vec() -> RVec<PluginFunction> {
    rvec![PluginFunction(parse_file)]
}

#[derive(Debug)]
pub struct ParseResult {
    pub file: go_parser::ast::File,
    pub ast_objects: go_parser::AstObjects,
}

impl Display for ParseResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&0, f)
    }
}

/// Appends a string to the erased `StringBuilder`.
#[sabi_extern_fn]
fn parse_file(
    pf_results: RVec<&mut BoxedPFResult<'_>>,
    user_params: RVec<&BoxedUserParam<'_>>,
) -> BoxedPFResult<'static> {
    dbg!(&pf_results);
    dbg!(&user_params);
    assert!(pf_results.is_empty());
    assert_eq!(user_params.len(), 1);

    let filepath = unsafe { user_params[0].unchecked_downcast_as::<RString>() };
    let source = std::fs::read_to_string(filepath.as_str()).expect("Failed to open go source file");

    let mut fs = go_parser::FileSet::new();
    let mut o = go_parser::AstObjects::new();
    let el = &mut go_parser::ErrorList::new();

    let (_parser, maybe_file) = go_parser::parse_file(&mut o, &mut fs, el, filepath, &source, true);

    DynTrait::from_value(ParseResult {
        file: maybe_file.expect("Something went wrong when trying to parse the source"),
        ast_objects: o,
    })
}

use abi_stable::{rvec, std_types::RVec};
use plugin_interface::{
    BoxedPFResult, BoxedUserParam, PFConnector, PFDependencies, PFType, Plugin, PluginRef,
    PluginFunction, QualPFID, UserParameters,
};
use std::fmt::{self, Display};

use abi_stable::{
    DynTrait, export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn, std_types::RString,
};

#[export_root_module]
pub fn get_library() -> PluginRef {
    Plugin { funcs: new_pf_vec }.leak_into_prefix()
}

#[sabi_extern_fn]
fn new_pf_vec() -> RVec<PFConnector> {
    rvec![PFConnector {
        pf: PluginFunction(parse_file),
        pf_type: PFType {
            pf_dependencies: rvec![],
            user_params: rvec!["file_path".into()]
        },
        pf_id: QualPFID {
            plugin_id: "plugin1".into(),
            pf_id: "parse_file".into()
        }
    }]
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
fn parse_file(pf_results: PFDependencies, user_params: &UserParameters) -> BoxedPFResult<'static> {
    dbg!(&pf_results);
    dbg!(&user_params);
    assert!(pf_results.is_empty());
    assert_eq!(user_params.len(), 1);

    let boxed_filepath: &BoxedUserParam = user_params.get("file_path").unwrap();
    let filepath = unsafe { boxed_filepath.unchecked_downcast_as::<RString>() };
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

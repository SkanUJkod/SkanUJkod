use abi_stable::{
    DynTrait, export_root_module, prefix_type::PrefixTypeTrait, rvec, sabi_extern_fn,
    std_types::RString, std_types::RVec,
};
use plugin_interface::{
    BoxedPFResult, BoxedUserParam, PFConnector, PFDependencies, PFType, Plugin, Plugin_Ref,
    PluginFunction, QualPFID, UserParameters,
};

use parse_dir_lib::{ParseDirDep, ParseDirParams};

#[export_root_module]
pub fn get_library() -> Plugin_Ref {
    Plugin { funcs: new_pf_vec }.leak_into_prefix()
}

#[sabi_extern_fn]
fn new_pf_vec() -> RVec<PFConnector> {
    rvec![PFConnector {
        pf: PluginFunction(parse_dir),
        pf_type: PFType {
            pf_dependencies: rvec![],
            user_params: rvec!["project_path".into()]
        },
        pf_id: QualPFID {
            plugin_id: "parse_dir_plugin".into(),
            pf_id: "parse_dir".into()
        }
    }]
}

/// Appends a string to the erased `StringBuilder`.
#[sabi_extern_fn]
fn parse_dir(pf_results: PFDependencies, user_params: &UserParameters) -> BoxedPFResult<'static> {
    dbg!(&pf_results);
    dbg!(&user_params);
    assert!(pf_results.is_empty());
    assert_eq!(user_params.len(), 1);

    let boxed_project_path: &BoxedUserParam = user_params.get("project_path").unwrap();
    let project_path = unsafe { boxed_project_path.unchecked_downcast_as::<RString>() };

    let result = parse_dir_lib::parse_dir(
        ParseDirDep {},
        &ParseDirParams {
            dir: project_path.clone(),
        },
    );

    DynTrait::from_value(result)
}
